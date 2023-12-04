mod common;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;

use crate::common::{setup_service, string_body};
use tower::ServiceExt;

#[tokio::test]
async fn header_guards() {
    let app = setup_service("./tests/header_guards");

    let response = app()
        .oneshot(
            Request::post("/system/mvp/route1")
                .header("USER", "success")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(string_body(response).await, "content");

    let response = app()
        .oneshot(
            Request::post("/system/mvp/route1")
                .header("USER", "error")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::from_u16(400).unwrap());
    assert_eq!(string_body(response).await, "error");
}
