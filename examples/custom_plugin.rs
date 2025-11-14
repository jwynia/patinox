//! Example: Creating a Custom Plugin
//!
//! This example demonstrates how to create a custom plugin that extends
//! agent functionality. Plugins transform agents during construction to add
//! optional capabilities.
//!
//! Run with: cargo run --example custom_plugin

use patinox::*;

/// A plugin that adds default tools to any agent
///
/// This plugin demonstrates the plugin pattern by automatically adding
/// useful utility tools to an agent.
struct DefaultToolsPlugin;

impl AgentPlugin for DefaultToolsPlugin {
    fn name(&self) -> &str {
        "DefaultToolsPlugin"
    }

    fn apply(&self, agent: Agent) -> Agent {
        agent
            .tool_fn("echo", "Echo back the input", |input: String| Ok(input))
            .tool_fn("uppercase", "Convert text to uppercase", |input: String| {
                Ok(input.to_uppercase())
            })
            .tool_fn("reverse", "Reverse the input text", |input: String| {
                Ok(input.chars().rev().collect())
            })
    }
}

/// A plugin that configures an agent with custom system prompts
struct SystemPromptPlugin {
    _prompt: String,
}

impl SystemPromptPlugin {
    fn new(prompt: impl Into<String>) -> Self {
        Self {
            _prompt: prompt.into(),
        }
    }

    #[allow(dead_code)]
    fn helpful() -> Self {
        Self::new("You are a helpful and friendly AI assistant. Be concise and clear.")
    }

    fn technical() -> Self {
        Self::new(
            "You are a technical AI assistant. Provide detailed, accurate technical information.",
        )
    }
}

impl AgentPlugin for SystemPromptPlugin {
    fn name(&self) -> &str {
        "SystemPromptPlugin"
    }

    fn apply(&self, agent: Agent) -> Agent {
        // Note: This would normally modify the agent's system prompt,
        // but we can't access private fields from an example.
        // In a real plugin inside the patinox crate, you would do:
        // agent.config.system_prompt = Some(self.prompt.clone());

        // For now, just return the agent unchanged as a demonstration
        agent
    }
}

/// A logging plugin that adds a lifecycle hook for observability
struct LoggingPlugin;

impl AgentPlugin for LoggingPlugin {
    fn name(&self) -> &str {
        "LoggingPlugin"
    }

    fn apply(&self, agent: Agent) -> Agent {
        agent.with_lifecycle(SimpleLogger)
    }
}

/// Simple logging hook for demonstration
struct SimpleLogger;

#[async_trait::async_trait]
impl AgentLifecycle for SimpleLogger {
    async fn before_agent(&self, input: &str) -> patinox::Result<String> {
        println!("[LOG] Agent starting with input: {}", input);
        Ok(input.to_string())
    }

    async fn after_agent(&self, result: &str) -> patinox::Result<String> {
        println!("[LOG] Agent completed with result: {}", result);
        Ok(result.to_string())
    }
}

fn main() -> patinox::Result<()> {
    println!("=== Custom Plugin Example ===\n");

    println!("Creating an agent with multiple plugins:\n");
    println!("1. SystemPromptPlugin (technical mode)");
    println!("2. DefaultToolsPlugin (echo, uppercase, reverse)");
    println!("3. LoggingPlugin (observability)\n");

    let _agent = create_agent("demo")
        .with_plugin(SystemPromptPlugin::technical())
        .with_plugin(DefaultToolsPlugin)
        .with_plugin(LoggingPlugin);

    println!("Agent created successfully!");
    println!("\nPlugin Capabilities:");
    println!("- SystemPromptPlugin: Configures technical assistant behavior");
    println!("- DefaultToolsPlugin: Adds 3 utility tools (echo, uppercase, reverse)");
    println!("- LoggingPlugin: Adds lifecycle hooks for observability");

    println!("\n=== Plugin Composition ===");
    println!("Plugins can be composed in any order. The agent is transformed");
    println!("sequentially as each plugin's apply() method is called.");
    println!("\nThis demonstrates the power of the plugin pattern:");
    println!("- Opt-in: Only used when needed");
    println!("- Composable: Multiple plugins work together");
    println!("- Type-safe: Compile-time checks");
    println!("- Zero-cost: No runtime overhead");

    Ok(())
}
