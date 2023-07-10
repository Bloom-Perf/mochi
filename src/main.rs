mod model;
mod extractor;

use std::convert::Infallible;
use std::fs;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use axum::body::Body;
use axum::http::{Method, Response};
use axum::routing::{delete, patch, put};
use serde_yaml::from_str;
use crate::extractor::{build_all};
use crate::model::yaml::{ApiShapeYaml, ApiYaml};

#[tokio::main]
async fn main() {
    // initialize tracing

    let document = fs::read_to_string("./config/sis/api-mvp.yml")
        .expect("Should have been able to read the file");
    let api: ApiYaml = from_str(&*document).unwrap();
    let document_shape = fs::read_to_string("./config/sis/api-shape-mvp.yml")
        .expect("Should have been able to read the file");
    let shape: ApiShapeYaml = from_str(&*document_shape).unwrap();

    let api_sets = build_all(vec![shape], vec![api]);

    // build our application with a route
    let mut app: Router = Router::new();

    for api_set in api_sets.into_iter() {
        for api in api_set.apis.into_iter() {
            for rule in api.rules.into_iter() {
                let create = move || async move {
                    let body = rule.body.map(|str| Body::from(str)).unwrap_or(Body::empty());
                    let body = axum::body::boxed(body);
                    let res = Response::builder()
                        .status(rule.status)
                        .body(body)
                        .unwrap();
                    Ok::<_, Infallible>(res)
                };


                app = app.route(
                    &*format!("/sis/{}{}", &api_set.name, rule.endpoint.route),
                    match rule.endpoint.method {
                        Method::GET => get(create),
                        Method::POST => post(create),
                        Method::PATCH => patch(create),
                        Method::PUT => put(create),
                        Method::DELETE => delete(create),
                        s => panic!("Unknown http method: {}", s)
                    }
                );
            }
        }
    }

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}