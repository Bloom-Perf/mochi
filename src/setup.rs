use anyhow::{Context, Error, Result};
use clap::Parser;
use log::info;
use mochi::setup_app;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[clap(flatten)]
    pub server: ServerConfig,
}

#[derive(Parser, Debug)]
pub struct ServerConfig {
    /// Path to configuration file
    #[clap(long, short, env = "CONFIG_PATH", default_value = "./config")]
    config_path: String,

    /// Port to listen on
    #[clap(long, short, env = "PORT", default_value = "3000")]
    port: u16,

    /// IPv4 address to bind to
    #[clap(long, short, env = "IP_ADDR", default_value = "0.0.0.0")]
    ip_addr: String,
}

pub async fn start_server(config: ServerConfig) -> Result<(), Error> {
    let app = setup_app(config.config_path).context("Failed to setup application")?;
    let ip: IpAddr = config
        .ip_addr
        .parse()
        .context("Failed to parse IP address")?;
    let addr = SocketAddr::new(ip, config.port);

    let tcp_listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;

    info!("Listening on: {}", addr);

    axum::serve(tcp_listener, app.into_make_service())
        .await
        .context("Failed to start HTTP server")
}
