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
    /// Comment text (for create/update operations)
    pub text: Option<String>,
    /// Parent comment ID (for threaded comments)
    pub parent_id: Option<String>,
    /// Assignee user ID (optional)
    pub assignee: Option<String>,
    /// Assigned commenter user ID (optional)
    pub assigned_commenter: Option<String>,
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
    /// List spaces in a workspace
    Spaces { workspace_id: String },
    /// List folders in a space
    Folders { space_id: String },
    /// List lists in a folder or space
    Lists { id: String, in_space: bool },
    /// Get a single task
    Task { task_id: String },
    /// Explore full hierarchy
    Explore { workspace_id: String },
    /// Get comments for a task
    Comments { task_id: String },
    /// Create a new comment on a task
    CreateComment { task_id: String },
    /// Create a reply to an existing comment
    CreateReply { comment_id: String },
    /// Update an existing comment
    UpdateComment { comment_id: String },
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
            text: None,
            parent_id: None,
            assignee: None,
            assigned_commenter: None,
        });
    }
    
    let mut operation: Option<DebugOperation> = None;
    let mut json = false;
    let mut verbose = false;
    let mut token_override: Option<String> = None;
    let mut text: Option<String> = None;
    let mut parent_id: Option<String> = None;
    let mut assignee: Option<String> = None;
    let mut assigned_commenter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "--text" => {
                if i + 1 >= args.len() {
                    return Err("--text requires a value".to_string());
                }
                text = Some(args[i + 1].clone());
                i += 1;
            }
            "--parent-id" => {
                if i + 1 >= args.len() {
                    return Err("--parent-id requires a value".to_string());
                }
                parent_id = Some(args[i + 1].clone());
                i += 1;
            }
            "--assignee" => {
                if i + 1 >= args.len() {
                    return Err("--assignee requires a value".to_string());
                }
                assignee = Some(args[i + 1].clone());
                i += 1;
            }
            "--assigned-commenter" => {
                if i + 1 >= args.len() {
                    return Err("--assigned-commenter requires a value".to_string());
                }
                assigned_commenter = Some(args[i + 1].clone());
                i += 1;
            }
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
            "spaces" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("spaces requires a workspace_id argument".to_string());
                }
                operation = Some(DebugOperation::Spaces {
                    workspace_id: args[i + 1].clone()
                });
                i += 1; // Skip next arg
            }
            "folders" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("folders requires a space_id argument".to_string());
                }
                operation = Some(DebugOperation::Folders {
                    space_id: args[i + 1].clone()
                });
                i += 1; // Skip next arg
            }
            "lists" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("lists requires a folder_id or space_id argument".to_string());
                }
                // Check for --in-space flag
                let mut in_space = false;
                if i + 2 < args.len() && args[i + 2] == "--in-space" {
                    in_space = true;
                }
                operation = Some(DebugOperation::Lists {
                    id: args[i + 1].clone(),
                    in_space,
                });
                i += 1; // Skip next arg
            }
            "task" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("task requires a task_id argument".to_string());
                }
                operation = Some(DebugOperation::Task {
                    task_id: args[i + 1].clone()
                });
                i += 1; // Skip next arg
            }
            "explore" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("explore requires a workspace_id argument".to_string());
                }
                operation = Some(DebugOperation::Explore {
                    workspace_id: args[i + 1].clone()
                });
                i += 1; // Skip next arg
            }
            "comments" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("comments requires a task_id argument".to_string());
                }
                operation = Some(DebugOperation::Comments {
                    task_id: args[i + 1].clone()
                });
                i += 1; // Skip next arg
            }
            "create-comment" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("create-comment requires a task_id argument".to_string());
                }
                operation = Some(DebugOperation::CreateComment {
                    task_id: args[i + 1].clone()
                });
                i += 1;
            }
            "create-reply" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("create-reply requires a comment_id argument".to_string());
                }
                operation = Some(DebugOperation::CreateReply {
                    comment_id: args[i + 1].clone()
                });
                i += 1;
            }
            "update-comment" => {
                if operation.is_some() {
                    return Err("Multiple operations specified".to_string());
                }
                if i + 1 >= args.len() {
                    return Err("update-comment requires a comment_id argument".to_string());
                }
                operation = Some(DebugOperation::UpdateComment {
                    comment_id: args[i + 1].clone()
                });
                i += 1;
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

    // Validate comment operation arguments
    match &op {
        DebugOperation::CreateComment { .. } | DebugOperation::CreateReply { .. } | DebugOperation::UpdateComment { .. } => {
            if text.is_none() {
                return Err("--text is required for this operation".to_string());
            }
            if text.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
                return Err("--text cannot be empty".to_string());
            }
        }
        _ => {}
    }

    Ok(DebugCommand {
        operation: op,
        json,
        verbose,
        token_override,
        text,
        parent_id,
        assignee,
        assigned_commenter,
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
    eprintln!("    spaces <workspace_id>   List spaces in a workspace");
    eprintln!("    folders <space_id>      List folders in a space");
    eprintln!("    lists <id>              List lists in a folder (use --in-space for space lists)");
    eprintln!("    task <task_id>          Get a single task");
    eprintln!("    comments <task_id>      Get comments for a task");
    eprintln!("    explore <workspace_id>  Explore full hierarchy (spaces->folders->lists->tasks)");
    eprintln!("    create-comment <task_id>  Create a new comment (--text required)");
    eprintln!("    create-reply <comment_id> Create a reply to a comment (--text required)");
    eprintln!("    update-comment <comment_id> Update an existing comment (--text required)");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    --json                  Output in JSON format");
    eprintln!("    --verbose, -v           Enable verbose logging");
    eprintln!("    --token <token>         Override stored token (for testing)");
    eprintln!("    --in-space              Use with 'lists' to list space lists instead of folder lists");
    eprintln!("    --text <text>           Comment text (for create/update operations)");
    eprintln!("    --parent-id <id>        Parent comment ID (for threaded comments)");
    eprintln!("    --assignee <user_id>    Assign comment to user");
    eprintln!("    --assigned-commenter <user_id>  Set who assigned the comment");
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
    eprintln!("    clickdown debug spaces 26408409 --json");
    eprintln!("    clickdown debug folders space123 --json");
    eprintln!("    clickdown debug lists folder123 --json");
    eprintln!("    clickdown debug task task123 --json");
    eprintln!("    clickdown debug comments task123 --json");
    eprintln!("    clickdown debug explore 26408409");
    eprintln!("    clickdown debug create-comment task123 --text \"Hello world\"");
    eprintln!("    clickdown debug create-reply comment456 --text \"Reply text\" --json");
    eprintln!("    clickdown debug update-comment comment789 --text \"Updated\" --verbose");
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
        let _args = vec!["debug".to_string(), "workspaces".to_string()];
        // Would need to mock env::args for full test
    }
}
