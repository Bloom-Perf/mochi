use axum::http::uri::PathAndQuery;
use axum::http::{Method, StatusCode, Uri};
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum LatencyCore {
    Constant(u32),
}

#[derive(Clone, Debug)]
pub struct ApiCore(pub Vec<RuleCore>);

#[derive(Clone, Debug)]
pub struct ProxyCore(pub Uri);

#[derive(Clone, Debug)]
pub struct ApiSetCore {
    pub name: String,
    pub shape: Option<Vec<EndpointCore>>,
    pub apis: Vec<ApiCore>,
    pub proxy: Option<ProxyCore>,
}

#[derive(Clone, Debug)]
pub struct ApiSetRootCore {
    pub shape: Option<Vec<EndpointCore>>,
    pub apis: Vec<ApiCore>,
    pub proxy: Option<ProxyCore>,
}

#[derive(Clone, Debug)]
pub struct RuleContentFeaturesCore {
    pub headers: bool,
    pub url_path: bool,
    pub url_query: bool,
}

#[derive(Clone, Debug)]
pub enum RuleBodyCore {
    Plain(String),
    Templated {
        headers: bool,
        url_path: bool,
        url_query: bool,
        request_body_json: bool,
        request_body_text: bool,
        registry: Handlebars<'static>,
    },
}

#[derive(Clone, Debug)]
pub struct RuleCore {
    pub endpoint: EndpointCore,
    pub headers: HashMap<String, String>,
    pub latency: Option<LatencyCore>,
    pub status: StatusCode,
    pub format: String,
    pub body: Option<RuleBodyCore>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EndpointCore {
    pub route: PathAndQuery,
    pub method: Method,
}

#[derive(Clone, Debug)]
pub struct SystemCore {
    pub name: String,
    pub root_api_set: ApiSetRootCore,
    pub api_sets: Vec<ApiSetCore>,
}

#[derive(Clone, Debug)]
pub struct ConfCore {
    pub systems: Vec<SystemCore>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HttpRoute {
    pub route: String,
    pub method: Method,
}

impl Display for EndpointCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.method, self.route.path())
    }
}
