use axum::http::uri::PathAndQuery;
use axum::http::{Method, StatusCode};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum LatencyCore {
    Constant(u32),
}

#[derive(Clone, Debug)]
pub struct ApiCore(pub Vec<RuleCore>);

#[derive(Clone, Debug)]
pub struct ApiSetCore {
    pub name: String,
    pub shape: Option<Vec<EndpointCore>>,
    pub apis: Vec<ApiCore>,
}

#[derive(Clone, Debug)]
pub struct RuleCore {
    pub endpoint: EndpointCore,
    pub headers: HashMap<String, String>,
    pub latency: Option<LatencyCore>,
    pub status: StatusCode,
    pub format: String,
    pub body: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EndpointCore {
    pub route: PathAndQuery,
    pub method: Method,
}

#[derive(Clone, Debug)]
pub struct SystemCore {
    pub name: String,
    pub api_sets: Vec<ApiSetCore>,
}
