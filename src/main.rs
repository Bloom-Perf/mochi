use anyhow::{Context, Result};
use clap::Parser;
use log::info;

mod logger;
mod setup;
use crate::logger::setup_logger;
use crate::setup::{start_server, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    setup_logger().context("Failed to setup logger")?;
    info!("Starting Mochi!");

    start_server(config.server).await?;
    Ok(())
}
