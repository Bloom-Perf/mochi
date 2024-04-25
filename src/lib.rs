use crate::http::routes::MochiRouterState;
use crate::yaml::from_files::ConfigurationFolder;
use anyhow::Result;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;

mod core;
mod http;
mod template;
mod yaml;

pub fn setup_app(conf_path: String) -> Result<Router<()>> {
    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    let core_representation = ConfigurationFolder::new(conf_path)
        .load_from_filesystem()?
        .extract()?;

    let initial_router = metrics_layer.routes::<MochiRouterState>();

    Ok(core_representation
        .build_router(initial_router)
        .layer(metrics_layer)
        .with_state(MochiRouterState::new()))
}
