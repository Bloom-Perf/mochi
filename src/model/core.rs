use axum::http::{HeaderName, Method, StatusCode};
use axum::http::uri::PathAndQuery;


#[derive(Clone, Debug)]
pub struct ApiCore {
    pub header: Option<HeaderName>,
    pub rules: Vec<RuleCore>
}

#[derive(Clone, Debug)]
pub struct ApiSetCore {
    pub name: String,
    pub shape: Option<Vec<EndpointCore>>,
    pub apis: Vec<ApiCore>
}

#[derive(Clone, Debug)]
pub struct RuleCore {
    pub endpoint: EndpointCore,
    pub status: StatusCode,
    pub format: String,
    pub body: Option<String>
}

#[derive(Clone, Debug)]
pub struct EndpointCore {
    pub route: PathAndQuery,
    pub method: Method,
}

#[derive(Clone, Debug)]
pub struct SystemCore {
    pub name: String,
    pub api_sets: Vec<ApiSetCore>
}