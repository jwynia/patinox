//! CLI interface for Patinox agents
//!
//! Provides command-line argument parsing and execution for agents.

use crate::Agent;
use std::env;
use std::io::{self, Read};

/// Run an agent with CLI interface
pub fn run_cli(agent: Agent) -> crate::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Handle special flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help(&agent);
                return Ok(());
            }
            "--version" | "-v" => {
                println!("{} v{}", agent.config.name, env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--tools" => {
                print_tools(&agent);
                return Ok(());
            }
            _ => {}
        }
    }

    // Get input from args or stdin
    let input = if args.len() > 1 {
        // Join all arguments after the program name
        args[1..].join(" ")
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    };

    if input.is_empty() {
        eprintln!("Error: No input provided");
        eprintln!(
            "Usage: {} <input>",
            args.first().unwrap_or(&"agent".to_string())
        );
        eprintln!(
            "   or: echo \"input\" | {}",
            args.first().unwrap_or(&"agent".to_string())
        );
        eprintln!("\nTry --help for more information");
        std::process::exit(1);
    }

    // Run the agent
    match agent.run(input) {
        Ok(output) => {
            println!("{}", output);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help(agent: &Agent) {
    println!("{}", agent.config.name);
    if let Some(desc) = &agent.config.description {
        println!("{}", desc);
    }
    println!();
    println!("USAGE:");
    println!("    {} <input>", agent.config.name);
    println!("    echo \"input\" | {}", agent.config.name);
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help message");
    println!("    -v, --version    Show version information");
    println!("    --tools          List available tools");
    println!();
    println!("EXAMPLES:");
    println!("    {} \"Hello, world!\"", agent.config.name);
    println!("    echo \"process this\" | {}", agent.config.name);
}

fn print_tools(agent: &Agent) {
    println!("Available tools:");
    if agent.tools.is_empty() {
        println!("  (none)");
    } else {
        for tool in agent.tools.values() {
            println!("  {} - {}", tool.name(), tool.description());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_agent;

    #[test]
    fn test_cli_help_doesnt_crash() {
        // Just ensure the help function doesn't crash
        let agent = create_agent("test");
        print_help(&agent);
    }

    #[test]
    fn test_cli_tools_list() {
        let agent =
            create_agent("test").tool_fn("hello", "Say hello", |_| Ok("Hello!".to_string()));
        print_tools(&agent);
    }
}
