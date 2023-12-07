mod common;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;

use crate::common::{setup_service, string_body};
use tower::ServiceExt;

#[tokio::test]
async fn multiple_routes() {
    let app = setup_service("./tests/multiple_routes");

    let response = app()
        .oneshot(Request::post("/system/route1").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(string_body(response).await, "content");

    let response = app()
        .oneshot(
            Request::patch("/system/route2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::from_u16(303).unwrap());
    assert_eq!(string_body(response).await, "{\n  \"test\": \"2\"\n}");
}
