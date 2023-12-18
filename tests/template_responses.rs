mod common;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use indoc::indoc;

use crate::common::{setup_service, string_body};
use tower::ServiceExt;

#[tokio::test]
async fn template_responses() {
    let app = setup_service("./tests/template_responses");

    let response = app()
        .oneshot(
            Request::post("/static/system/route/my_path_param?foo=myfoo")
                .header("header", "my simple header")
                .body(Body::from("{\"test\": \"test_value1234\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        string_body(response).await,
        indoc!(
            r###"{
              "static": "2",
              "headers.header": "my simple header",
              "url.query.foo": "myfoo",
              "url.path.path_param": "my_path_param",
              "unknown parameter url.test.test": "",
              "body json param": "test_value1234"
            }"###
        )
    );
}

#[tokio::test]
async fn xpath() {
    let app = setup_service("./tests/template_responses");

    let response = app()
        .oneshot(
            Request::post("/static/system/xpath")
                .body(Body::from(
                    r###"<node>
    <el name="v1" />
    <el name="v2" />
    <el name="v3" />
</node>"###,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        string_body(response).await,
        indoc!(
            r###"<node>
  <len>1</len>
  <results>
    <n>v1</n>
  </results>
</node>"###
        )
    );
}

#[tokio::test]
async fn xpath_str() {
    let app = setup_service("./tests/template_responses");

    let response = app()
        .oneshot(
            Request::post("/static/system/xpath_str")
                .body(Body::from(
                    r###"<node>
    <el name="v1">hello</el>
    <el name="v2" />
    <el name="v3">world</el>
</node>"###,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        string_body(response).await,
        indoc!(
            r###"<node>
  <len>1</len>
  <results>
    <n>hello</n>
  </results>
</node>"###
        )
    );
}
