//! ClickDown - A TUI ClickUp client

#![allow(dead_code)]
#![allow(unused_imports)]

mod api;
mod models;
mod cache;
mod config;
mod tui;
mod cli;
mod commands;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    // Parse CLI arguments
    let args = cli::args::parse_args().map_err(|e| {
        eprintln!("Error parsing arguments: {}", e);
        cli::args::print_usage();
        anyhow::anyhow!("Invalid arguments")
    })?;

    match args.debug_command {
        Some(cmd) => {
            // Run in CLI debug mode
            tracing::info!("Running in CLI debug mode");
            let exit_code = cli::run::run_cli(cmd).await;
            std::process::exit(exit_code);
        }
        None => {
            // Run in TUI mode
            tracing::info!("Starting ClickDown TUI...");
            let mut app = tui::app::TuiApp::new()?;
            app.run()?;
        }
    }

    Ok(())
}
