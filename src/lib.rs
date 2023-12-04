use crate::metrics::MochiMetrics;
use crate::yaml::from_files::ConfigurationFolder;
use anyhow::Result;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;

mod core;
mod http;
mod metrics;
mod yaml;

pub fn setup_app(conf_path: String) -> Result<Router<()>> {
    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    let core_representation = ConfigurationFolder::new(conf_path)
        .load_from_filesystem()?
        .extract()?;

    let mochi_metrics = MochiMetrics::create();
    let initial_router = metrics_layer.routes::<MochiMetrics>();

    Ok(core_representation
        .build_router(initial_router)
        .layer(metrics_layer)
        .with_state(mochi_metrics.clone()))
}
