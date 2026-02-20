//! ClickDown - A fast ClickUp desktop client

#![allow(dead_code)]
#![allow(unused_imports)]

mod api;
mod models;
mod cache;
mod config;
mod ui;
mod app;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}

fn main() -> Result<()> {
    init_logging();
    
    tracing::info!("Starting ClickDown...");
    
    app::run()?;
    
    Ok(())
}
