//! CLI execution logic
//!
//! Handles running the CLI debug mode operations.

use std::sync::Arc;

use crate::api::{ClickUpApi, ClickUpClient, AuthManager};
use crate::cli::args::{DebugCommand, DebugOperation, exit_codes};
use crate::commands::DebugOperations;

/// Run the CLI with the given arguments
/// Returns the exit code as an i32
pub async fn run_cli(command: DebugCommand) -> i32 {
    // Handle help first (no auth needed)
    if matches!(command.operation, DebugOperation::Help) {
        crate::cli::args::print_usage();
        return exit_codes::SUCCESS;
    }

    // Set up logging if verbose
    if command.verbose {
        // Only set if RUST_LOG is not already set
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug");
        }
        tracing::info!("Verbose mode enabled");
    }

    // Log token override (without showing the token)
    if command.token_override.is_some() {
        tracing::info!("Using override token");
    }

    // Initialize auth manager
    let auth = AuthManager::default();

    // Get token (use override if provided)
    let token = match command.token_override.clone() {
        Some(override_token) => override_token,
        None => match auth.load_token() {
            Ok(Some(t)) => t,
            Ok(None) => {
                eprintln!("No API token found. Please set your ClickUp API token.");
                eprintln!("Token location: ~/.config/clickdown/token");
                return exit_codes::AUTH_ERROR;
            }
            Err(e) => {
                eprintln!("Failed to read token: {}", e);
                return exit_codes::AUTH_ERROR;
            }
        },
    };

    // Create API client
    let client = ClickUpClient::new(token);
    let api: Arc<dyn ClickUpApi> = Arc::new(client);

    // Create debug operations handler
    let debug_ops = DebugOperations::new(api, auth, command.token_override.clone());

    // Execute the operation
    let result = match command.operation {
        DebugOperation::Workspaces => {
            if command.json {
                debug_ops.list_workspaces_json().await
            } else {
                debug_ops.list_workspaces().await
            }
        }
        DebugOperation::Tasks { ref list_id } => {
            if command.json {
                debug_ops.list_tasks_json(list_id).await
            } else {
                debug_ops.list_tasks(list_id).await
            }
        }
        DebugOperation::Docs { ref query } => {
            if command.json {
                debug_ops.search_docs_json(query).await
            } else {
                debug_ops.search_docs(query).await
            }
        }
        DebugOperation::AuthStatus => {
            match debug_ops.check_auth_status().await {
                Ok(true) => return exit_codes::SUCCESS,
                Ok(false) => return exit_codes::AUTH_ERROR,
                Err(e) => {
                    eprintln!("Error checking auth status: {}", e);
                    return exit_codes::GENERAL_ERROR;
                }
            }
        }
        DebugOperation::Help => {
            // Already handled above
            return exit_codes::SUCCESS;
        }
    };

    // Handle result
    match result {
        Ok(()) => exit_codes::SUCCESS,
        Err(e) => {
            let err_msg = e.to_string();
            
            // Determine exit code based on error type
            let exit_code = if err_msg.contains("authentication") 
                || err_msg.contains("unauthorized")
                || err_msg.contains("401")
                || err_msg.contains("403") {
                exit_codes::AUTH_ERROR
            } else if err_msg.contains("network")
                || err_msg.contains("connection")
                || err_msg.contains("timeout") {
                exit_codes::NETWORK_ERROR
            } else if err_msg.contains("not found")
                || err_msg.contains("404") {
                exit_codes::GENERAL_ERROR
            } else {
                exit_codes::GENERAL_ERROR
            };

            if command.verbose {
                eprintln!("Error: {:?}", e);
            } else {
                eprintln!("Error: {}", e);
            }

            exit_code
        }
    }
}
