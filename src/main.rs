use crate::logger::setup_logger;

use anyhow::{Context, Result};
use log::info;
use mochi::setup_app;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod logger;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().context("Setup logger")?;

    info!("Starting Mochi!");

    let configpath = env::var("CONFIG_PATH").unwrap_or("./config".to_string());

    let app = setup_app(configpath)?;

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let tcp_listener = TcpListener::bind(addr).await?;

    axum::serve(tcp_listener, app.into_make_service())
        .await
        .context("Starting http server")
}
