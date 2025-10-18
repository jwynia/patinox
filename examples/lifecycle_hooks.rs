//! Example demonstrating lifecycle hooks
//!
//! This example shows how to use lifecycle hooks to add middleware behavior
//! to agents without modifying the core agent logic.

use async_trait::async_trait;
use patinox::*;

/// A simple logging hook that tracks agent execution
struct LoggingHook {
    name: String,
}

impl LoggingHook {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl AgentLifecycle for LoggingHook {
    async fn before_agent(&self, input: &str) -> Result<String> {
        println!("[{}] Agent starting with input: {}", self.name, input);
        Ok(input.to_string())
    }

    async fn before_model(
        &self,
        messages: Vec<patinox::provider::Message>,
    ) -> Result<Vec<patinox::provider::Message>> {
        println!(
            "[{}] Sending {} messages to model",
            self.name,
            messages.len()
        );
        Ok(messages)
    }

    async fn after_agent(&self, result: &str) -> Result<String> {
        println!("[{}] Agent completed with result: {}", self.name, result);
        Ok(result.to_string())
    }
}

/// A hook that transforms input to uppercase
struct UppercaseHook;

#[async_trait]
impl AgentLifecycle for UppercaseHook {
    async fn before_agent(&self, input: &str) -> Result<String> {
        println!("[UppercaseHook] Transforming input to uppercase");
        Ok(input.to_uppercase())
    }
}

/// A hook that can reject responses based on content
struct ContentFilterHook;

#[async_trait]
impl AgentLifecycle for ContentFilterHook {
    async fn after_model(
        &self,
        response: &patinox::provider::ProviderResponse,
    ) -> Result<HookAction> {
        use patinox::provider::ProviderResponse;

        match response {
            ProviderResponse::Text(text) => {
                if text.contains("forbidden") {
                    println!("[ContentFilterHook] Rejecting response with forbidden content");
                    Ok(HookAction::Reject(
                        "Response contains forbidden content".to_string(),
                    ))
                } else {
                    println!("[ContentFilterHook] Response approved");
                    Ok(HookAction::Continue)
                }
            }
            _ => Ok(HookAction::Continue),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Lifecycle Hooks Example ===\n");

    // Example 1: Single logging hook
    println!("Example 1: Single logging hook");
    println!("---");

    let agent = create_agent("example-1")
        .with_provider(Box::new(patinox::provider::MockProvider::new(
            "Hello from the agent!",
        )))
        .with_lifecycle(LoggingHook::new("Logger"));

    let result = agent.run("test input").await?;
    println!("Result: {}\n", result);

    // Example 2: Multiple chained hooks
    println!("Example 2: Multiple chained hooks (logging + uppercase transform)");
    println!("---");

    let agent = create_agent("example-2")
        .with_provider(Box::new(patinox::provider::MockProvider::new(
            "Response to uppercase input",
        )))
        .with_lifecycle(LoggingHook::new("Logger"))
        .with_lifecycle(UppercaseHook);

    let result = agent.run("hello world").await?;
    println!("Result: {}\n", result);

    // Example 3: Content filtering hook
    println!("Example 3: Content filtering hook (rejecting forbidden content)");
    println!("---");

    let agent = create_agent("example-3")
        .with_provider(Box::new(patinox::provider::MockProvider::new(
            "This response contains forbidden words",
        )))
        .with_lifecycle(LoggingHook::new("Logger"))
        .with_lifecycle(ContentFilterHook);

    match agent.run("test").await {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("❌ Request rejected: {}", e),
    }

    println!("\n=== End of Examples ===");

    Ok(())
}
