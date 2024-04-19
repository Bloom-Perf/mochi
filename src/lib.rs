use crate::metrics::MochiMetrics;
use crate::yaml::from_files::ConfigurationFolder;
use anyhow::Result;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use http_body_util::BodyExt;
use itertools::{repeat_n, Itertools};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

mod core;
mod http;
mod metrics;
mod template;
mod yaml;

#[derive(Debug)]
pub struct ProxyState {
    pub routes: Vec<NodePath>,
}

impl ProxyState {
    pub fn new() -> ProxyState {
        ProxyState { routes: vec![] }
    }
    pub fn append_path(&mut self, path: &Vec<String>) {
        let mut root = &mut self.routes;

        for p in path.iter() {
            // if let Some(found) = root.iter().find_or_first(|n| p.eq(&n.value.value)) {
            //     root = &found.children;
            // } else {
            //     let node = NodePath::constant(p.clone());
            //     let children = &node.children;
            //     root.push(node);
            //     root = children;
            // }

            let next_node_idx = ProxyState::get_child_idx(&root, p);

            root = match next_node_idx {
                Some(x) => &mut root[x].children,
                None => {
                    let new_node = NodePath::constant(p.clone());
                    root.push(new_node);
                    &mut root.last_mut().unwrap().children
                }
            }
        }
    }

    fn get_child_idx<'a>(v: &Vec<NodePath>, curr_c: &String) -> Option<usize> {
        v.iter()
            .enumerate()
            .find(|(_, n)| n.value.eq(curr_c))
            .map(|(i, _)| i)
    }
}

#[derive(Debug)]
struct PathChunk {
    value: String,
}

impl PathChunk {}

#[derive(Debug)]
struct NodePath {
    pub value: String,
    pub children: Vec<NodePath>,
}

impl NodePath {
    pub fn constant(str: String) -> NodePath {
        NodePath {
            value: str,
            children: vec![],
        }
    }

    pub fn display(&self, offset: usize) -> String {
        format!(
            "{}{}\n{}",
            repeat_n(" -> ", offset).into_iter().format(""),
            &self.value,
            self.children
                .iter()
                .map(|c| c.display(offset + 1))
                .format("")
        )
    }
}

#[derive(Clone)]
pub struct MochiRouterState {
    pub metrics: MochiMetrics,
    pub proxy: Arc<RwLock<ProxyState>>,
}

pub fn setup_app(conf_path: String) -> Result<Router<()>> {
    let metrics_layer = HttpMetricsLayerBuilder::new()
        .with_service_name("mochi".to_ascii_uppercase())
        .build();

    let core_representation = ConfigurationFolder::new(conf_path)
        .load_from_filesystem()?
        .extract()?;

    let mochi_metrics = MochiMetrics::create();
    let proxy_state = ProxyState::new();
    let mochi_router_state = MochiRouterState {
        metrics: mochi_metrics,
        proxy: Arc::new(RwLock::new(proxy_state)),
    };
    let initial_router = metrics_layer.routes::<MochiRouterState>();

    Ok(core_representation
        .build_router(initial_router)
        .layer(metrics_layer)
        .with_state(mochi_router_state))
}
