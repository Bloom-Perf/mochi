use crate::logger::setup_logger;
use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use mochi::setup_app;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;

mod logger;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_logger().context("Setup logger")?;
    info!("Starting Mochi!");

    let app = setup_app(args.config_path)?;
    let ip: IpAddr = args.ip_addr.parse().context("Invalid IP address")?;
    let addr = SocketAddr::new(ip, args.port);
    let tcp_listener = TcpListener::bind(addr).await?;

    info!("Listening on: {}", addr);

    axum::serve(tcp_listener, app.into_make_service())
        .await
        .context("Starting http server")
}
