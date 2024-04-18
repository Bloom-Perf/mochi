use crate::metrics::MochiMetrics;
use crate::yaml::from_files::ConfigurationFolder;
use anyhow::Result;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;

mod core;
mod http;
mod metrics;
mod template;
mod yaml;

#[derive(Clone)]
pub struct ProxyState {}

#[derive(Clone)]
pub struct MochiRouterState {
    pub metrics: MochiMetrics,
    pub proxy: ProxyState,
}

pub fn setup_app(conf_path: String) -> Result<Router<()>> {
    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    let core_representation = ConfigurationFolder::new(conf_path)
        .load_from_filesystem()?
        .extract()?;

    let mochi_metrics = MochiMetrics::create();
    let proxy_state = ProxyState {};
    let mochi_router_state = MochiRouterState {
        metrics: mochi_metrics,
        proxy: proxy_state,
    };
    let initial_router = metrics_layer.routes::<MochiRouterState>();

    Ok(core_representation
        .build_router(initial_router)
        .layer(metrics_layer)
        .with_state(mochi_router_state))
}
