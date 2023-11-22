mod extractor;
mod file;
mod logger;
mod metrics;
mod model;
mod routes;

use crate::extractor::build_all;
use crate::file::ConfigurationFolder;
use crate::logger::setup_logger;
use crate::model::core::SystemCore;
use crate::model::yaml::SystemFolder;
use crate::routes::handle_request;

use crate::metrics::MochiMetrics;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{on, MethodFilter};
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use log::{info, warn};
use std::env;
use std::net::SocketAddr;

async fn handler404(request: Request<Body>) -> Response {
    warn!(
        "Request with route --- \n\t[{}] {}\n --- did not match any route of the configuration",
        request.method(),
        request.uri()
    );
    StatusCode::NOT_FOUND.into_response()
}
#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().context("Setup logger")?;

    let metrics = HttpMetricsLayerBuilder::new().build();

    info!("Starting Mochi!");

    let configpath = env::var("CONFIG_PATH").unwrap_or("./config".to_string());

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new(configpath).load_systems();

    let rules_maps: Vec<_> = system_folders
        .into_iter()
        .map(|system| {
            let s = SystemCore {
                name: system.name.to_owned(),
                api_sets: build_all(system.shapes.to_owned(), system.apis, system.data).unwrap(),
            };
            s.generate_rules_map()
        })
        .collect();

    let mochi_metrics = MochiMetrics {};
    let initial_router = metrics.routes::<MochiMetrics>();

    let app = rules_maps
        .into_iter()
        .fold(initial_router, move |r, map| {
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
                .fallback(handler404);
            r.merge(subrouter)
        })
        .layer(metrics)
        .with_state(mochi_metrics.clone());

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Starting http server")
}
