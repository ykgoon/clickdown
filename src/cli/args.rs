//! CLI argument parsing for debug mode
//!
//! Handles parsing of command-line arguments for the debug subcommand.

use std::env;

/// Exit codes for CLI operations
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const INVALID_ARGS: i32 = 2;
    pub const AUTH_ERROR: i32 = 3;
    pub const NETWORK_ERROR: i32 = 4;
}

/// Parsed CLI arguments
#[derive(Debug, Clone)]
pub struct CliArgs {
    /// The debug operation to run (if any)
    pub debug_command: Option<DebugCommand>,
}

/// Debug subcommand structure
#[derive(Debug, Clone)]
pub struct DebugCommand {
    /// The operation to perform
    pub operation: DebugOperation,
    /// Output as JSON
    pub json: bool,
    /// Enable verbose logging
    pub verbose: bool,
    /// Override token (for testing)
    pub token_override: Option<String>,
}

/// Available debug operations
#[derive(Debug, Clone, PartialEq)]
pub enum DebugOperation {
    /// List all workspaces
    Workspaces,
    /// List tasks from a list
    Tasks { list_id: String },
    /// Search documents
    Docs { query: String },
    /// Check authentication status
    AuthStatus,
    /// Show help
    Help,
}

/// Parse CLI arguments from environment
pub fn parse_args() -> Result<CliArgs, String> {
    let args: Vec<String> = env::args().collect();
    
    // Skip program name
    if args.len() < 2 {
        // No subcommand - run TUI mode
        return Ok(CliArgs { debug_command: None });
    }
    
    let subcommand = &args[1];
    
    match subcommand.as_str() {
        "debug" => {
            let debug_cmd = parse_debug_command(&args[2..])?;
            Ok(CliArgs { debug_command: Some(debug_cmd) })
        }
        "--help" | "-h" | "help" => {
            // Show help and run TUI
            Ok(CliArgs { debug_command: None })
        }
        _ => {
            Err(format!("Unknown subcommand: {}", subcommand))
        }
    }
}

/// Parse the debug subcommand arguments
fn parse_debug_command(args: &[String]) -> Result<DebugCommand, String> {
    if args.is_empty() {
        // No operation specified - show help
        return Ok(DebugCommand {
            operation: DebugOperation::Help,
            json: false,
            verbose: false,
            token_override: None,
        });
    }
    
    let mut operation: Option<DebugOperation> = None;
    let mut json = false;
    let mut verbose = false;
    let mut token_override: Option<String> = None;
    
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        
        match arg.as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "--token" => {
                if i + 1 >= args.len() {
                    return Err("--token requires a value".to_string());
                }
                token_override = Some(args[i + 1].clone());
                i += 1; // Skip next arg
            }
            "workspaces" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                operation = Some(DebugOperation::Workspaces);
            }
            "tasks" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("tasks requires a list_id argument".to_string());
                }
                operation = Some(DebugOperation::Tasks { 
                    list_id: args[i + 1].clone() 
                });
                i += 1; // Skip next arg
            }
            "docs" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("docs requires a query argument".to_string());
                }
                operation = Some(DebugOperation::Docs { 
                    query: args[i + 1].clone() 
                });
                i += 1; // Skip next arg
            }
            "auth-status" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                operation = Some(DebugOperation::AuthStatus);
            }
            "--help" | "-h" => {
                operation = Some(DebugOperation::Help);
            }
            _ => {
                // Check if it looks like an operation
                if operation.is_none() && !arg.starts_with('-') {
                    return Err(format!("Unknown operation: {}", arg));
                } else if arg.starts_with('-') {
                    return Err(format!("Unknown option: {}", arg));
                }
            }
        }
        i += 1;
    }
    
    let op = operation.unwrap_or(DebugOperation::Help);
    
    Ok(DebugCommand {
        operation: op,
        json,
        verbose,
        token_override,
    })
}

/// Display usage information
pub fn print_usage() {
    eprintln!("ClickDown - Terminal-based ClickUp client");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    clickdown                    Run in TUI mode");
    eprintln!("    clickdown debug <operation> [OPTIONS]");
    eprintln!();
    eprintln!("DEBUG OPERATIONS:");
    eprintln!("    workspaces              List all authorized workspaces");
    eprintln!("    tasks <list_id>         Fetch tasks from a list");
    eprintln!("    docs <query>            Search documents");
    eprintln!("    auth-status             Check authentication status");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    --json                  Output in JSON format");
    eprintln!("    --verbose, -v           Enable verbose logging");
    eprintln!("    --token <token>         Override stored token (for testing)");
    eprintln!("    --help, -h              Show this help message");
    eprintln!();
    eprintln!("EXIT CODES:");
    eprintln!("    0   Success");
    eprintln!("    1   General error");
    eprintln!("    2   Invalid arguments");
    eprintln!("    3   Authentication error");
    eprintln!("    4   Network error");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    clickdown debug workspaces");
    eprintln!("    clickdown debug tasks list123 --json");
    eprintln!("    clickdown debug auth-status --verbose");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_args() {
        // Simulating no args would return TUI mode
        // This is tested via env::args() behavior
    }

    #[test]
    fn test_parse_debug_workspaces() {
        let args = vec!["debug".to_string(), "workspaces".to_string()];
        // Would need to mock env::args for full test
    }
}
