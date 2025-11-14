//! Documentation Generator Agent - V2 Real Usage Example #2
//!
//! This agent reads Rust source code and generates markdown documentation.
//! It's the second real-world usage of the V2 framework to validate plugin needs.
//!
//! Build: cargo build --example doc_generator --release
//! Run: OPENAI_API_KEY=sk-... ./target/release/examples/doc_generator <rust-file>
//!
//! Examples:
//!   doc_generator src/agent.rs
//!   doc_generator src/tool.rs --output docs/tool.md
//!   doc_generator src/ --recursive

use patinox::prelude::*;
use patinox::provider::{OpenAIProvider, ProviderConfig};
use std::fs;
use std::path::Path;

fn main() -> patinox::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage(&args[0]);
        return Ok(());
    }

    // Get source file path
    let source_path = &args[1];

    // Check for output flag
    let output_path = if let Some(idx) = args.iter().position(|a| a == "--output" || a == "-o") {
        args.get(idx + 1).map(|s| s.to_string())
    } else {
        None
    };

    // Build the query for the LLM
    let user_query =
        "Please generate comprehensive markdown documentation for this Rust source code. \
         Include:\n\
         - Purpose and overview\n\
         - Main types and their roles\n\
         - Key functions and methods\n\
         - Usage examples\n\
         - Important notes or considerations\n\n\
         Format the output as a clean markdown document suitable for a docs/ folder."
            .to_string();

    // Create agent with documentation tools
    // Note: Using ToolContextExt to eliminate clone + move boilerplate
    let mut agent = create_agent("doc_generator")
        .tool_fn_with("read_source", "Read Rust source file", source_path, |path, _args| {
            read_source_tool(path)
        })
        .tool_fn_with(
            "get_module_info",
            "Get information about the module",
            source_path,
            |path, _args| get_module_info_tool(path),
        )
        .tool_fn_with(
            "extract_public_api",
            "Extract public API items",
            source_path,
            |path, _args| extract_public_api_tool(path),
        )
        .tool_fn_with(
            "count_functions",
            "Count functions in the source",
            source_path,
            |path, _args| count_functions_tool(path),
        );

    // Add write tool if output path specified
    if let Some(ref out_path) = output_path {
        agent = agent.tool_fn_with(
            "write_documentation",
            "Write documentation to file",
            out_path,
            |path, content| write_documentation_tool(path, content),
        );
    }

    // Set up OpenAI provider
    let config = ProviderConfig::new(Provider::OpenAI)
        .model("gpt-4o-mini")
        .temperature(0.7)
        .max_tokens(2000); // Higher token limit for documentation

    match OpenAIProvider::new(config) {
        Ok(provider) => {
            agent = agent.with_provider(Box::new(provider));
            println!("✓ Using OpenAI provider (gpt-4o-mini)");
            println!("✓ Generating documentation for: {}\n", source_path);
            if let Some(ref out) = output_path {
                println!("✓ Output will be written to: {}\n", out);
            }
        }
        Err(e) => {
            eprintln!("⚠ Error: Could not initialize OpenAI provider: {}", e);
            eprintln!("⚠ Make sure OPENAI_API_KEY is set in your environment");
            std::process::exit(1);
        }
    }

    // Run agent
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        match agent.run(user_query).await {
            Ok(output) => {
                println!("\n{}", output);

                // If output path specified, write to file
                if let Some(out_path) = output_path {
                    match fs::write(&out_path, &output) {
                        Ok(_) => {
                            println!("\n✓ Documentation written to: {}", out_path);
                        }
                        Err(e) => {
                            eprintln!("\n⚠ Warning: Could not write to file: {}", e);
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("\nError generating documentation: {}", e);
                std::process::exit(1);
            }
        }
    })
}

fn print_usage(program_name: &str) {
    println!("Documentation Generator - Generate docs from Rust source code");
    println!();
    println!("USAGE:");
    println!("    {} <source-file> [OPTIONS]", program_name);
    println!();
    println!("ARGUMENTS:");
    println!("    <source-file>    Path to Rust source file (.rs)");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output <file>    Write documentation to file");
    println!("    -h, --help             Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    {} src/agent.rs", program_name);
    println!("    {} src/tool.rs -o docs/tool.md", program_name);
    println!(
        "    {} src/provider.rs --output docs/provider.md",
        program_name
    );
    println!();
    println!("ENVIRONMENT:");
    println!("    OPENAI_API_KEY    Required: Your OpenAI API key");
}

// Tool implementations

fn read_source_tool(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("Failed to read source file '{}': {}", path, e).into()),
    }
}

fn get_module_info_tool(path: &str) -> ToolResult {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("Source file does not exist: {}", path).into());
    }

    let file_name = path_obj
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let extension = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    if extension != "rs" {
        return Err(format!("Not a Rust source file: {}", path).into());
    }

    // Try to extract module doc comment
    let module_doc = match fs::read_to_string(path) {
        Ok(contents) => {
            // Look for //! comments at the start
            let doc_lines: Vec<_> = contents
                .lines()
                .take_while(|line| line.trim().starts_with("//!") || line.trim().is_empty())
                .filter(|line| line.trim().starts_with("//!"))
                .map(|line| line.trim_start_matches("//!").trim())
                .collect();

            if doc_lines.is_empty() {
                "No module documentation found".to_string()
            } else {
                doc_lines.join("\n")
            }
        }
        Err(_) => "Could not read module documentation".to_string(),
    };

    Ok(format!(
        "File: {}\nType: Rust source file\nModule documentation:\n{}",
        file_name, module_doc
    ))
}

fn extract_public_api_tool(path: &str) -> ToolResult {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read source file '{}': {}", path, e))?;

    // Simple extraction of public items (pub fn, pub struct, pub enum, pub trait)
    let mut public_items = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn ") {
            if let Some(name) = extract_item_name(trimmed, "pub fn ") {
                public_items.push(format!("fn {}", name));
            }
        } else if trimmed.starts_with("pub struct ") {
            if let Some(name) = extract_item_name(trimmed, "pub struct ") {
                public_items.push(format!("struct {}", name));
            }
        } else if trimmed.starts_with("pub enum ") {
            if let Some(name) = extract_item_name(trimmed, "pub enum ") {
                public_items.push(format!("enum {}", name));
            }
        } else if trimmed.starts_with("pub trait ") {
            if let Some(name) = extract_item_name(trimmed, "pub trait ") {
                public_items.push(format!("trait {}", name));
            }
        } else if trimmed.starts_with("pub type ") {
            if let Some(name) = extract_item_name(trimmed, "pub type ") {
                public_items.push(format!("type {}", name));
            }
        } else if trimmed.starts_with("pub mod ") {
            if let Some(name) = extract_item_name(trimmed, "pub mod ") {
                public_items.push(format!("mod {}", name));
            }
        }
    }

    if public_items.is_empty() {
        Ok("No public API items found".to_string())
    } else {
        Ok(format!(
            "Public API items ({} found):\n{}",
            public_items.len(),
            public_items.join("\n")
        ))
    }
}

fn extract_item_name(line: &str, prefix: &str) -> Option<String> {
    line.strip_prefix(prefix).and_then(|rest| {
        // Extract up to first space, <, or (
        rest.split(&['<', '(', ' ', '{'][..])
            .next()
            .map(|s| s.to_string())
    })
}

fn count_functions_tool(path: &str) -> ToolResult {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read source file '{}': {}", path, e))?;

    let mut fn_count = 0;
    let mut pub_fn_count = 0;
    let mut impl_count = 0;
    let mut test_count = 0;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") {
            fn_count += 1;
        } else if trimmed.starts_with("pub fn ") {
            pub_fn_count += 1;
        } else if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
            impl_count += 1;
        } else if trimmed.contains("#[test]") || trimmed.contains("#[tokio::test]") {
            test_count += 1;
        }
    }

    Ok(format!(
        "Function counts:\n\
         - Total functions: {}\n\
         - Public functions: {}\n\
         - Impl blocks: {}\n\
         - Test functions: {}",
        fn_count, pub_fn_count, impl_count, test_count
    ))
}

fn write_documentation_tool(path: &str, content: String) -> ToolResult {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
        }
    }

    fs::write(path, content)
        .map_err(|e| format!("Failed to write documentation to '{}': {}", path, e))?;

    Ok(format!("Documentation written to: {}", path))
}
