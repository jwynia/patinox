//! Hello Agent - Minimal Patinox Example
//!
//! This is the simplest possible Patinox agent, demonstrating the minimal API.
//!
//! Build: cargo build --example hello_agent --release
//! Run: ./target/release/examples/hello_agent "world"

use patinox::prelude::*;
use patinox::provider::MockProvider;

fn main() -> patinox::Result<()> {
    // Create an agent with a simple greeting tool
    let agent = create_agent("hello")
        .tool_fn("greet", "Say hello to someone", |name| {
            Ok(format!("Hello, {}!", name))
        })
        .tool_fn("uppercase", "Convert text to uppercase", |text| {
            Ok(text.to_uppercase())
        })
        .tool_fn("count_words", "Count words in text", |text| {
            let count = text.split_whitespace().count();
            Ok(format!("Word count: {}", count))
        })
        // Use mock provider for testing (no API key needed)
        .with_provider(Box::new(MockProvider::new(
            "I used the greet tool to say: Hello, world!"
        )));

    // Run with CLI interface
    agent.run_cli()
}
