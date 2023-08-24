mod extractor;
mod file;
mod model;
mod routes;

use crate::extractor::build_all;
use crate::file::ConfigurationFolder;
use crate::model::core::SystemCore;
use crate::model::yaml::SystemFolder;
use crate::routes::handle_request;
use axum::body::Body;
use axum::http::Request;
use axum::routing::{get, on, MethodFilter};
use axum::Router;
use axum_prometheus::PrometheusMetricLayer;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let configpath = env::var("CONFIG_PATH").unwrap_or("./config".to_string());
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new(configpath).load_systems();

    let rules_maps: Vec<_> = system_folders
        .into_iter()
        .map(|system| {
            let s = SystemCore {
                name: system.name.to_owned(),
                api_sets: build_all(system.shapes.to_owned(), system.apis, system.data),
            };
            s.generate_rules_map()
        })
        .collect();

    let app: Router = rules_maps
        .into_iter()
        .fold(Router::new(), move |r, map| {
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
                });
            r.merge(subrouter)
        })
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
