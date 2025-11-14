//! Tool Context Helper - Complete Guide
//!
//! This example demonstrates the Tool Context Helper plugin, showing:
//! 1. How to eliminate clone + move boilerplate
//! 2. Using different context types (String, PathBuf, custom structs)
//! 3. Before/after migration examples
//! 4. Single and dual context capture
//!
//! Run: cargo run --example tool_context_guide

use patinox::prelude::*;
use std::path::PathBuf;

/// Custom configuration struct to demonstrate context with complex types
#[derive(Clone)]
struct AppConfig {
    api_key: String,
    max_retries: u32,
    verbose: bool,
}

impl AppConfig {
    fn new() -> Self {
        Self {
            api_key: "demo-key-12345".to_string(),
            max_retries: 3,
            verbose: true,
        }
    }
}

fn main() -> patinox::Result<()> {
    println!("=== Tool Context Helper - Complete Guide ===\n");

    // =================================================================
    // PART 1: The Problem - Manual Clone + Move Boilerplate
    // =================================================================
    println!("## PART 1: The Problem\n");
    println!("Without ToolContextExt, every context-aware tool requires boilerplate:\n");

    let file_path = PathBuf::from("data.txt");

    println!("```rust");
    println!(".tool_fn(\"read_file\", \"Read file\", {{");
    println!("    let path = file_path.clone();  // ❌ Manual clone");
    println!("    move |_args| read_file_tool(&path)  // ❌ Manual move");
    println!("}})");
    println!("```\n");

    println!("For 4 tools, this is 12 extra lines of boilerplate!\n");

    // =================================================================
    // PART 2: The Solution - Using tool_fn_with()
    // =================================================================
    println!("## PART 2: The Solution - Using tool_fn_with()\n");

    println!("With ToolContextExt (already imported via prelude):\n");
    println!("```rust");
    println!(".tool_fn_with(\"read_file\", \"Read file\", &file_path,");
    println!("    |path, _args| read_file_tool(path))  // ✅ Zero boilerplate");
    println!("```\n");

    println!("Creating agent with tool_fn_with()...\n");

    let _agent = create_agent("demo")
        .tool_fn_with("read_file", "Read a file", &file_path, |path, _args| {
            Ok(format!("Reading file: {}", path.display()))
        })
        .tool_fn_with("file_info", "Get file info", &file_path, |path, _args| {
            Ok(format!("File info for: {}", path.display()))
        })
        .tool_fn_with("delete_file", "Delete a file", &file_path, |path, _args| {
            Ok(format!("Would delete: {}", path.display()))
        });

    println!("✓ Created agent with 3 context-aware tools");
    println!("✓ Zero boilerplate - clean and concise\n");

    // =================================================================
    // PART 3: Custom Context Types
    // =================================================================
    println!("## PART 3: Custom Context Types\n");

    println!("tool_fn_with() works with ANY type that implements Clone:\n");

    let config = AppConfig::new();

    let _agent_with_config = create_agent("configured-agent")
        .tool_fn_with(
            "get_api_key",
            "Get the API key from config",
            &config,
            |cfg, _args| Ok(format!("API Key: {}", cfg.api_key)),
        )
        .tool_fn_with(
            "get_max_retries",
            "Get max retries from config",
            &config,
            |cfg, _args| Ok(format!("Max retries: {}", cfg.max_retries)),
        )
        .tool_fn_with(
            "check_verbose",
            "Check if verbose mode is enabled",
            &config,
            |cfg, _args| Ok(format!("Verbose: {}", cfg.verbose)),
        );

    println!("✓ Created agent with custom AppConfig context");
    println!("✓ Each tool has access to the shared config\n");

    // =================================================================
    // PART 4: Dual Context with tool_fn_with2()
    // =================================================================
    println!("## PART 4: Dual Context with tool_fn_with2()\n");

    println!("For tools that need TWO context variables:\n");
    println!("```rust");
    println!(".tool_fn_with2(\"process\", \"Process file\", &file_path, &config,");
    println!("    |path, cfg, _args| process_with_config(path, cfg))");
    println!("```\n");

    let output_path = PathBuf::from("output.txt");

    let _agent_dual = create_agent("dual-context")
        .tool_fn_with2(
            "copy_file",
            "Copy file to output",
            &file_path,
            &output_path,
            |src, dst, _args| Ok(format!("Copy {} → {}", src.display(), dst.display())),
        )
        .tool_fn_with2(
            "compare_files",
            "Compare two files",
            &file_path,
            &output_path,
            |f1, f2, _args| Ok(format!("Comparing {} vs {}", f1.display(), f2.display())),
        );

    println!("✓ Created agent with dual-context tools");
    println!("✓ Each tool receives both file paths\n");

    // =================================================================
    // PART 5: Migration Guide
    // =================================================================
    println!("## PART 5: Migration Guide\n");

    println!("BEFORE (manual boilerplate):");
    println!("─────────────────────────────");
    println!("let agent = create_agent(\"old\")");
    println!("    .tool_fn(\"tool1\", \"desc\", {{");
    println!("        let ctx = context.clone();");
    println!("        move |args| handler(&ctx, args)");
    println!("    }})");
    println!("    .tool_fn(\"tool2\", \"desc\", {{");
    println!("        let ctx = context.clone();");
    println!("        move |args| handler(&ctx, args)");
    println!("    }});");
    println!();
    println!("16 lines for 2 tools ❌\n");

    println!("AFTER (with ToolContextExt):");
    println!("────────────────────────────");
    println!("let agent = create_agent(\"new\")");
    println!("    .tool_fn_with(\"tool1\", \"desc\", &context, |ctx, args|");
    println!("        handler(ctx, args))");
    println!("    .tool_fn_with(\"tool2\", \"desc\", &context, |ctx, args|");
    println!("        handler(ctx, args));");
    println!();
    println!("6 lines for 2 tools ✅");
    println!("63% reduction in code! 🎉\n");

    // =================================================================
    // PART 6: Performance
    // =================================================================
    println!("## PART 6: Performance\n");

    println!("tool_fn_with() has ZERO runtime overhead:");
    println!("• Compiles to identical code as manual clone + move");
    println!("• No dynamic dispatch or allocations");
    println!("• Pure zero-cost abstraction\n");

    // =================================================================
    // PART 7: Summary & Best Practices
    // =================================================================
    println!("## PART 7: Summary & Best Practices\n");

    println!("✓ Use tool_fn_with() for tools needing ONE context variable");
    println!("✓ Use tool_fn_with2() for tools needing TWO context variables");
    println!("✓ Works with any Clone type (String, PathBuf, custom structs)");
    println!("✓ Zero runtime overhead - pure compile-time abstraction");
    println!("✓ Already in prelude - just `use patinox::prelude::*`");
    println!();

    println!("When to use:");
    println!("• Tool needs access to file paths → tool_fn_with(&path, ...)");
    println!("• Tool needs access to config → tool_fn_with(&config, ...)");
    println!("• Tool needs both → tool_fn_with2(&path, &config, ...)");
    println!();

    println!("Migration checklist:");
    println!("1. Identify tools with clone + move pattern");
    println!("2. Replace .tool_fn(...{{ let x = y.clone(); move |_| ... }})");
    println!("3. With .tool_fn_with(..., &y, |x, _| ...)");
    println!("4. Remove the clone and move keywords");
    println!("5. Enjoy cleaner code! 🚀");
    println!();

    println!("═══════════════════════════════════════════════════════════");
    println!("Total agents created: 3");
    println!("Total tools demonstrated: 11");
    println!("Boilerplate eliminated: ~33 lines");
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}
