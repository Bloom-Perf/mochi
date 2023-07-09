mod model;
mod extractor;

use std::convert::Infallible;
use std::fs;
use std::future::Future;
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::body::Body;
use axum::handler::Handler;
use axum::http::{Method, Response};
use axum::routing::{delete, patch, put};
use serde_yaml::from_str;
use crate::extractor::Extractor;
use crate::model::yaml::ApiYaml;

#[tokio::main]
async fn main() {
    // initialize tracing

    let document = fs::read_to_string("./config/sis/api-mvp.yml")
        .expect("Should have been able to read the file");

    let parsed: ApiYaml = from_str(&*document).unwrap();
    let api = parsed.extract().unwrap();

    // build our application with a route
    let mut app: Router = Router::new();

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
            &*format!("/sis/{}{}", &parsed.name, rule.route),
            match rule.method {
                Method::GET => get(create),
                Method::POST => post(create),
                Method::PATCH => patch(create),
                Method::PUT => put(create),
                Method::DELETE => delete(create),
                s => panic!("Unknown http method: {}", s)
            }
        );
    }

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}