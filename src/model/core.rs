use axum::http::{HeaderName, Method, StatusCode};
use axum::http::uri::PathAndQuery;

pub struct ApiCore {
    pub header: Option<HeaderName>,
    pub rules: Vec<RuleCore>
}

pub struct ApiSetCore {
    pub name: String,
    pub shape: Option<Vec<EndpointCore>>,
    pub apis: Vec<ApiCore>
}

pub struct RuleCore {
    pub endpoint: EndpointCore,
    pub status: StatusCode,
    pub format: String,
    pub body: Option<String>
}

pub struct EndpointCore {
    pub route: PathAndQuery,
    pub method: Method,
}