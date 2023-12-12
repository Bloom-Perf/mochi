mod common;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;

use crate::common::{setup_service, string_body};
use tower::ServiceExt;

#[tokio::test]
async fn basic() {
    let app = setup_service("./tests/basic");

    let response = app()
        .oneshot(
            Request::post("/static/system/route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(string_body(response).await, "content");
}
