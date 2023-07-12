mod model;
mod extractor;
mod file;

use std::convert::Infallible;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use axum::body::Body;
use axum::http::{Method, Response};
use axum::routing::{delete, patch, put};
use crate::extractor::{build_all};
use crate::file::ConfigurationFolder;
use crate::model::core::SystemCore;
use crate::model::yaml::{SystemFolder};

#[tokio::main]
async fn main() {
    // initialize tracing

    let system_folders: Vec<SystemFolder> = ConfigurationFolder::new("./config").load_systems();

    let systems: Vec<SystemCore> = system_folders
        .into_iter()
        .map(|system| SystemCore {
            name: system.name,
            api_sets: build_all(system.shapes, system.apis)
        }).collect();

    // build our application with a route
    let mut app: Router = Router::new();

    for system in systems.into_iter() {
        for api_set in system.api_sets.into_iter() {
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
                        &*format!("/{}/{}{}", &*system.name, &api_set.name, rule.endpoint.route),
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
    }

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}