use axum::body::Body;
use axum::response::Response;
use axum::routing::RouterIntoService;
use http_body_util::BodyExt;
use mochi::setup_app;

pub async fn string_body(response: Response) -> String {
    let bytes = response.into_body().collect().await.unwrap();
    
    String::from_utf8(bytes.to_bytes().to_vec()).unwrap()
}

pub fn setup_service(path: &'static str) -> Box<dyn Fn() -> RouterIntoService<Body>> {
    Box::new(move || setup_app(path.to_string()).unwrap().into_service())
}
