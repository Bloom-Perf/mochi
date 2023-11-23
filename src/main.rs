mod core;
mod http;
mod logger;
mod metrics;
mod yaml;

use crate::core::SystemCore;
use crate::http::routes::handle_request;
use crate::logger::setup_logger;
use crate::yaml::from_files::ConfigurationFolder;
use crate::yaml::to_domain::build_all;
use crate::yaml::SystemFolder;

use crate::metrics::MochiMetrics;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{on, MethodFilter};
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use log::{info, warn};
use std::env;
use std::net::SocketAddr;

async fn handler404(
    State(metrics): State<MochiMetrics>,
    request: Request<Body>,
    system_name: String,
) -> Response {
    warn!(
        "Request with route --- \n\t[{}] {}\n --- did not match any route of the configuration of system \"{}\"",
        request.method(),
        request.uri(),
        system_name
    );
    metrics.mochi_route_not_found(system_name);
    StatusCode::NOT_FOUND.into_response()
}
#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().context("Setup logger")?;

    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    info!("Starting Mochi!");

    let configpath = env::var("CONFIG_PATH").unwrap_or("./config".to_string());

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new(configpath).load_systems();

    let rules_maps: Vec<_> = system_folders
        .into_iter()
        .map(|system| {
            let system_name = system.name.clone();
            let s = SystemCore {
                name: system.name.to_owned(),
                api_sets: build_all(system.shapes.to_owned(), system.apis, system.data).unwrap(),
            };
            (system_name, s.generate_rules_map())
        })
        .collect();

    let mochi_metrics = MochiMetrics::create();
    let initial_router = metrics_layer.routes::<MochiMetrics>();

    let app = rules_maps
        .into_iter()
        .fold(initial_router, move |r, (system_name, map)| {
            let system_name_subrouter = system_name.clone();
            let subrouter = map
                .into_iter()
                .fold(Router::new(), |acc, (endpoint, rules)| {
                    // dbg!(endpoint.clone());
                    acc.route(
                        &endpoint.route,
                        on(MethodFilter::try_from(endpoint.clone().method).unwrap(), {
                            move |request: Request<Body>| handle_request(request, rules.to_owned())
                        }),
                    )
                })
                .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
                    handler404(m, r, system_name_subrouter)
                });
            r.nest(&format!("/{}", system_name), subrouter)
        })
        .fallback(move |m: State<MochiMetrics>, r: Request<Body>| {
            handler404(m, r, "Mochi System".to_string())
        })
        .layer(metrics_layer)
        .with_state(mochi_metrics.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Starting http server")
}
