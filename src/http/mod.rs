use crate::MochiRouterState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use log::warn;

pub mod proxy;
pub mod routes;

pub async fn handler404(
    State(s): State<MochiRouterState>,
    request: Request<Body>,
    system_name: String,
) -> Response {
    warn!(
        "Request with route --- \n\t[{}] {}\n --- did not match any route of the configuration of system \"{}\"",
        request.method(),
        request.uri(),
        system_name
    );
    s.metrics.mochi_route_not_found(system_name);
    StatusCode::NOT_FOUND.into_response()
}
