mod core;
mod http;
mod logger;
mod metrics;
mod yaml;

use crate::logger::setup_logger;
use crate::yaml::from_files::ConfigurationFolder;
use crate::yaml::ConfFolder;

use crate::metrics::MochiMetrics;
use anyhow::{Context, Result};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use log::info;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().context("Setup logger")?;

    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    info!("Starting Mochi!");

    let configpath = env::var("CONFIG_PATH").unwrap_or("./config".to_string());

    let conf_folder: ConfFolder = ConfigurationFolder::new(configpath).load_from_filesystem();

    let repr = conf_folder.extract()?;

    let mochi_metrics = MochiMetrics::create();
    let initial_router = metrics_layer.routes::<MochiMetrics>();

    let app = repr
        .build_router(initial_router)
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
