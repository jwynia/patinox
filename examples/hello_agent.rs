//! Hello Agent - Minimal Patinox Example
//!
//! This is the simplest possible Patinox agent, demonstrating the minimal API.
//!
//! Build: cargo build --example hello_agent --release
//! Run: OPENAI_API_KEY=sk-... ./target/release/examples/hello_agent "Say hello to the world"
//!
//! Or use mock provider for testing without an API key:
//! Run: cargo run --example hello_agent --features mock "test input"

use patinox::prelude::*;
use patinox::provider::{OpenAIProvider, ProviderConfig};

fn main() -> patinox::Result<()> {
    // Create an agent with a simple greeting tool
    let mut agent = create_agent("hello")
        .tool_fn("greet", "Say hello to someone", |name| {
            Ok(format!("Hello, {}!", name))
        })
        .tool_fn("uppercase", "Convert text to uppercase", |text| {
            Ok(text.to_uppercase())
        })
        .tool_fn("count_words", "Count words in text", |text| {
            let count = text.split_whitespace().count();
            Ok(format!("Word count: {}", count))
        });

    // Use real OpenAI provider (requires OPENAI_API_KEY environment variable)
    let config = ProviderConfig::new(Provider::OpenAI)
        .model("gpt-4o-mini")
        .temperature(0.7)
        .max_tokens(150);

    match OpenAIProvider::new(config) {
        Ok(provider) => {
            agent = agent.with_provider(Box::new(provider));
            println!("✓ Using OpenAI provider (gpt-4o-mini)");
        }
        Err(e) => {
            eprintln!("⚠ Warning: Could not initialize OpenAI provider: {}", e);
            eprintln!("⚠ Make sure OPENAI_API_KEY is set in your environment");
            eprintln!("⚠ Example: export OPENAI_API_KEY=sk-...");
            std::process::exit(1);
        }
    }

    // Run with CLI interface
    agent.run_cli()
}
