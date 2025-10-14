//! Agent core implementation
//!
//! The Agent is the central orchestrator that combines tools, providers,
//! and execution logic into a working AI agent.

use crate::provider::{LLMProvider, Message, Provider, ProviderConfig, ProviderResponse, ToolDefinition};
use crate::tool::Tool;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Agent configuration
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub provider_config: ProviderConfig,
}

impl AgentConfig {
    /// Create a new agent config with default provider (Anthropic)
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            system_prompt: Some("You are a helpful AI assistant.".to_string()),
            provider_config: ProviderConfig::new(Provider::Anthropic),
        }
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the provider
    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider_config = ProviderConfig::new(provider);
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.provider_config = self.provider_config.model(model);
        self
    }
}

/// Agent - the core orchestrator
pub struct Agent {
    pub(crate) config: AgentConfig,
    pub(crate) tools: HashMap<String, Arc<dyn Tool>>,
    provider: Option<Box<dyn LLMProvider>>,
}

impl Agent {
    /// Create a new agent with configuration
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            tools: HashMap::new(),
            provider: None,
        }
    }

    /// Add a tool to the agent
    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
        self
    }

    /// Add a tool from a closure (convenience method)
    pub fn tool_fn<F>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(String) -> crate::tool::ToolResult + Send + Sync + 'static,
    {
        use crate::tool::FnTool;
        let tool = FnTool::from_string_fn(name, description, handler);
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
        self
    }

    /// Set a custom provider (for testing or custom implementations)
    pub fn with_provider(mut self, provider: Box<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Run the agent with a single input
    pub async fn run(&self, input: impl Into<String>) -> crate::Result<String> {
        let provider = self
            .provider
            .as_ref()
            .map(|p| p.as_ref())
            .unwrap_or_else(|| {
                panic!(
                    "No provider configured. Use with_provider() or set up environment variables."
                );
            });

        // Build initial messages
        let mut messages = Vec::new();

        if let Some(sys_prompt) = &self.config.system_prompt {
            messages.push(Message::system(sys_prompt));
        }

        messages.push(Message::user(input.into()));

        // Convert tools to ToolDefinitions
        let tool_defs: Vec<ToolDefinition> = self
            .tools
            .values()
            .map(|tool| ToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            })
            .collect();

        // Tool calling loop (max 10 iterations to prevent infinite loops)
        let max_iterations = 10;
        for iteration in 0..max_iterations {
            // Get completion from provider
            let response = provider.complete(messages.clone(), tool_defs.clone()).await?;

            match response {
                ProviderResponse::Text(text) => {
                    // Final response - return it
                    return Ok(text);
                }
                ProviderResponse::ToolCalls(calls) => {
                    // Execute each tool call
                    for call in calls {
                        let tool = self.tools.get(&call.name).ok_or_else(|| {
                            format!("Tool '{}' not found", call.name)
                        })?;

                        // Execute the tool
                        let result = tool.execute(call.arguments)?;

                        // Add tool result to messages
                        // For simplicity, we add it as an assistant message
                        messages.push(Message::assistant(format!(
                            "Tool '{}' returned: {}",
                            call.name, result
                        )));
                    }
                }
            }

            // If we got here, we had tool calls and need to continue the loop
            if iteration == max_iterations - 1 {
                return Err("Max tool calling iterations reached".into());
            }
        }

        Err("Tool calling loop ended unexpectedly".into())
    }

    /// Run the agent with CLI interface
    pub fn run_cli(self) -> crate::Result<()> {
        crate::cli::run_cli(self)
    }
}

/// Helper function to create an agent
pub fn create_agent(name: impl Into<String>) -> Agent {
    Agent::new(AgentConfig::new(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;

    #[test]
    fn test_agent_creation() {
        let agent = create_agent("test");
        assert_eq!(agent.config.name, "test");
    }

    #[test]
    fn test_agent_with_tool() {
        let agent = create_agent("test")
            .tool_fn("hello", "Say hello", |name| Ok(format!("Hello, {}!", name)));

        assert!(agent.tools.contains_key("hello"));
    }

    #[tokio::test]
    async fn test_agent_with_mock_provider() {
        let agent =
            create_agent("test").with_provider(Box::new(MockProvider::new("test response")));

        let result = agent.run("hello").await.unwrap();
        assert_eq!(result, "test response");
    }

    #[test]
    fn test_agent_config_builder() {
        let config = AgentConfig::new("test")
            .description("A test agent")
            .system_prompt("Custom prompt")
            .provider(Provider::OpenAI)
            .model("gpt-4o");

        assert_eq!(config.name, "test");
        assert_eq!(config.description, Some("A test agent".to_string()));
        assert_eq!(config.provider_config.model, "gpt-4o");
    }
}
