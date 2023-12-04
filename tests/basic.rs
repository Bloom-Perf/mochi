use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;

use http_body_util::BodyExt;
use mochi::setup_app;
use tower::ServiceExt;

async fn string_body(response: Response) -> String {
    let bytes = response.into_body().collect().await.unwrap();
    let body_str = String::from_utf8(bytes.to_bytes().to_vec()).unwrap();
    body_str
}
#[tokio::test]
async fn test() {
    let app = setup_app("./tests/fixtures/basic".to_string())
        .unwrap()
        .into_service();

    let response = app
        .oneshot(
            Request::post("/system/mvp/route")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(string_body(response).await, "content");
}
