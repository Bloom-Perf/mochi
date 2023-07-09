use axum::body::Body;
use axum::http::{Method, StatusCode};
use axum::http::uri::PathAndQuery;

pub struct ApiCore {
    pub name: String,
    pub header: Option<(String, String)>,
    pub rules: Vec<RuleCore>
}

pub struct RuleCore {
    pub route: PathAndQuery,
    pub method: Method,
    pub status: StatusCode,
    pub format: String,
    pub body: Option<String>
}