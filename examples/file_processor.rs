//! File Processor Agent - V2 Real Usage Example
//!
//! This agent processes text files with LLM analysis. It's the first real-world
//! usage of the V2 agent framework to identify what plugins are actually needed.
//!
//! Build: cargo build --example file_processor --release
//! Run: OPENAI_API_KEY=sk-... ./target/release/examples/file_processor <file-path>
//!
//! Examples:
//!   file_processor README.md "Summarize this file"
//!   file_processor src/agent.rs "What does this code do?"
//!   file_processor --analyze CLAUDE.md

use patinox::prelude::*;
use patinox::provider::{OpenAIProvider, ProviderConfig};
use std::fs;
use std::path::Path;

fn main() -> patinox::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    // Check for special flags
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage(&args[0]);
        return Ok(());
    }

    // Get file path from first argument
    let file_path = &args[1];

    // Get user query (optional, defaults to analysis request)
    let user_query = if args.len() > 2 {
        args[2..].join(" ")
    } else {
        format!("Please analyze the file '{}' and provide insights about its content, structure, and purpose.", file_path)
    };

    // Create agent with file processing tools
    // Note: Using ToolContextExt to eliminate clone + move boilerplate
    let mut agent = create_agent("file_processor")
        .tool_fn_with("read_file", "Read the contents of a file", file_path, |path, _args| {
            read_file_tool(path)
        })
        .tool_fn_with("count_lines", "Count lines in a file", file_path, |path, _args| {
            count_lines_tool(path)
        })
        .tool_fn_with(
            "get_file_info",
            "Get file metadata (size, type, etc.)",
            file_path,
            |path, _args| get_file_info_tool(path),
        )
        .tool_fn_with(
            "extract_keywords",
            "Extract keywords from file content",
            file_path,
            |path, _args| extract_keywords_tool(path),
        );

    // Set up OpenAI provider
    let config = ProviderConfig::new(Provider::OpenAI)
        .model("gpt-4o-mini")
        .temperature(0.7)
        .max_tokens(1000);

    match OpenAIProvider::new(config) {
        Ok(provider) => {
            agent = agent.with_provider(Box::new(provider));
            println!("✓ Using OpenAI provider (gpt-4o-mini)");
            println!("✓ Processing file: {}\n", file_path);
        }
        Err(e) => {
            eprintln!("⚠ Error: Could not initialize OpenAI provider: {}", e);
            eprintln!("⚠ Make sure OPENAI_API_KEY is set in your environment");
            std::process::exit(1);
        }
    }

    // Create tokio runtime and run agent
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        match agent.run(user_query).await {
            Ok(output) => {
                println!("\n{}", output);
                Ok(())
            }
            Err(e) => {
                eprintln!("\nError running agent: {}", e);
                std::process::exit(1);
            }
        }
    })
}

fn print_usage(program_name: &str) {
    println!("File Processor Agent - Analyze files with AI");
    println!();
    println!("USAGE:");
    println!("    {} <file-path> [query]", program_name);
    println!();
    println!("ARGUMENTS:");
    println!("    <file-path>    Path to the file to process");
    println!("    [query]        Optional: What you want to know about the file");
    println!("                   Default: General analysis of the file");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help     Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    {} README.md", program_name);
    println!(
        "    {} src/main.rs \"What does this code do?\"",
        program_name
    );
    println!("    {} data.txt \"Summarize this file\"", program_name);
    println!();
    println!("ENVIRONMENT:");
    println!("    OPENAI_API_KEY    Required: Your OpenAI API key");
}

// Tool implementations

fn read_file_tool(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("Failed to read file '{}': {}", path, e).into()),
    }
}

fn count_lines_tool(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let line_count = contents.lines().count();
            let word_count = contents.split_whitespace().count();
            let char_count = contents.len();
            Ok(format!(
                "Lines: {}\nWords: {}\nCharacters: {}",
                line_count, word_count, char_count
            ))
        }
        Err(e) => Err(format!("Failed to read file '{}': {}", path, e).into()),
    }
}

fn get_file_info_tool(path: &str) -> ToolResult {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path).into());
    }

    let metadata =
        fs::metadata(path).map_err(|e| format!("Failed to get metadata for '{}': {}", path, e))?;

    let file_type = if metadata.is_file() {
        "File"
    } else if metadata.is_dir() {
        "Directory"
    } else {
        "Other"
    };

    let extension = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("(none)");

    let size_bytes = metadata.len();
    let size_kb = size_bytes as f64 / 1024.0;

    Ok(format!(
        "Path: {}\nType: {}\nExtension: {}\nSize: {} bytes ({:.2} KB)",
        path, file_type, extension, size_bytes, size_kb
    ))
}

fn extract_keywords_tool(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(contents) => {
            // Simple keyword extraction: find words longer than 4 chars, count frequency
            let mut word_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for word in contents.split_whitespace() {
                let cleaned = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                if cleaned.len() > 4 {
                    *word_counts.entry(cleaned).or_insert(0) += 1;
                }
            }

            // Get top 10 keywords
            let mut sorted: Vec<_> = word_counts.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));

            let top_keywords: Vec<String> = sorted
                .iter()
                .take(10)
                .map(|(word, count)| format!("{} ({})", word, count))
                .collect();

            if top_keywords.is_empty() {
                Ok("No significant keywords found".to_string())
            } else {
                Ok(format!("Top keywords:\n{}", top_keywords.join("\n")))
            }
        }
        Err(e) => Err(format!("Failed to read file '{}': {}", path, e).into()),
    }
}
