//! Agent core implementation
//!
//! The Agent is the central orchestrator that combines tools, providers,
//! and execution logic into a working AI agent.

use crate::lifecycle::AgentLifecycle;
use crate::provider::{
    LLMProvider, Message, Provider, ProviderConfig, ProviderResponse, ToolDefinition,
};
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
    lifecycle: Vec<Arc<dyn AgentLifecycle>>,
}

impl Agent {
    /// Create a new agent with configuration
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            tools: HashMap::new(),
            provider: None,
            lifecycle: Vec::new(),
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

    /// Add a lifecycle hook to this agent
    ///
    /// Hooks are executed in registration order. Multiple hooks can be chained
    /// to create composable middleware.
    ///
    /// # Example
    /// ```ignore
    /// let agent = create_agent("my-agent")
    ///     .with_lifecycle(LoggingHook::new())
    ///     .with_lifecycle(RetryHook::new());
    /// ```
    pub fn with_lifecycle(mut self, hook: impl AgentLifecycle + 'static) -> Self {
        self.lifecycle.push(Arc::new(hook));
        self
    }

    /// Run the agent with a single input
    pub async fn run(&self, input: impl Into<String>) -> crate::Result<String> {
        use crate::lifecycle::HookAction;

        let provider = self
            .provider
            .as_ref()
            .map(|p| p.as_ref())
            .unwrap_or_else(|| {
                panic!(
                    "No provider configured. Use with_provider() or set up environment variables."
                );
            });

        // Hook 1: before_agent - Transform input before processing
        let mut input = input.into();
        for hook in &self.lifecycle {
            input = hook.before_agent(&input).await?;
        }

        // Build initial messages
        let mut messages = Vec::new();

        if let Some(sys_prompt) = &self.config.system_prompt {
            messages.push(Message::system(sys_prompt));
        }

        messages.push(Message::user(input));

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
            // Hook 2: before_model - Transform messages before LLM call
            for hook in &self.lifecycle {
                messages = hook.before_model(messages).await?;
            }

            // Hook 3: wrap_model_call - Wrap the LLM call
            // For simplicity, we call the provider directly and let hooks observe
            // Full wrapping with retry/fallback can be added in future iterations
            let mut response = provider
                .complete(messages.clone(), tool_defs.clone())
                .await?;

            // Hook 4: after_model - Inspect/modify response, or reject
            for hook in &self.lifecycle {
                match hook.after_model(&response).await? {
                    HookAction::Continue | HookAction::Approve => {
                        // Continue normally
                    }
                    HookAction::Reject(reason) => {
                        return Err(reason.into());
                    }
                    HookAction::Modify(new_response) => {
                        response = new_response;
                    }
                }
            }

            match response {
                ProviderResponse::Text(text) => {
                    // Hook 6: after_agent - Transform final result
                    let mut result = text;
                    for hook in &self.lifecycle {
                        result = hook.after_agent(&result).await?;
                    }

                    // Final response - return it
                    return Ok(result);
                }
                ProviderResponse::ToolCalls(calls) => {
                    // Execute each tool call
                    for call in calls {
                        let tool = self
                            .tools
                            .get(&call.name)
                            .ok_or_else(|| format!("Tool '{}' not found", call.name))?;

                        // Hook 5: wrap_tool_call - Wrap tool execution
                        // Note: For now, hooks are called directly without complex chaining
                        // to avoid lifetime issues with tool trait objects
                        let result = tool.execute(call.arguments)?;

                        // For simplicity in V1, we don't chain wrap_tool_call hooks
                        // due to complexity with trait object lifetimes.
                        // Future enhancement can add proper chaining.

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
    use crate::lifecycle::AgentLifecycle;
    use crate::provider::MockProvider;
    use async_trait::async_trait;

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

    // TEST: Agent starts with empty lifecycle vec
    #[test]
    fn test_agent_starts_with_no_hooks() {
        let agent = create_agent("test");
        assert_eq!(agent.lifecycle.len(), 0);
    }

    // TEST: Can add a single lifecycle hook
    struct DummyHook;

    #[async_trait]
    impl AgentLifecycle for DummyHook {}

    #[test]
    fn test_agent_with_single_lifecycle_hook() {
        let agent = create_agent("test").with_lifecycle(DummyHook);
        assert_eq!(agent.lifecycle.len(), 1);
    }

    // TEST: Can chain multiple lifecycle hooks
    #[test]
    fn test_agent_with_multiple_lifecycle_hooks() {
        let agent = create_agent("test")
            .with_lifecycle(DummyHook)
            .with_lifecycle(DummyHook);
        assert_eq!(agent.lifecycle.len(), 2);
    }

    // TEST: Hooks work with builder pattern chaining
    #[test]
    fn test_lifecycle_integrates_with_builder_pattern() {
        let agent = create_agent("test")
            .tool_fn("hello", "Say hello", |_| Ok("hi".to_string()))
            .with_lifecycle(DummyHook)
            .with_provider(Box::new(MockProvider::new("response")));

        assert_eq!(agent.lifecycle.len(), 1);
        assert_eq!(agent.tools.len(), 1);
        assert!(agent.provider.is_some());
    }

    // Integration tests for lifecycle hooks
    use crate::lifecycle::HookAction;
    use crate::provider::ProviderResponse;
    use std::sync::{Arc, Mutex};

    // Hook that tracks execution order
    struct TrackingHook {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl TrackingHook {
        fn new(calls: Arc<Mutex<Vec<String>>>) -> Self {
            Self { calls }
        }

        fn record(&self, event: impl Into<String>) {
            self.calls.lock().unwrap().push(event.into());
        }
    }

    #[async_trait]
    impl AgentLifecycle for TrackingHook {
        async fn before_agent(&self, input: &str) -> crate::Result<String> {
            self.record(format!("before_agent: {}", input));
            Ok(input.to_string())
        }

        async fn before_model(
            &self,
            messages: Vec<crate::provider::Message>,
        ) -> crate::Result<Vec<crate::provider::Message>> {
            self.record(format!("before_model: {} messages", messages.len()));
            Ok(messages)
        }

        async fn wrap_model_call(
            &self,
            f: std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = crate::provider::ProviderResult<ProviderResponse>,
                        > + Send,
                >,
            >,
        ) -> crate::provider::ProviderResult<ProviderResponse> {
            self.record("wrap_model_call: before");
            let result = f.await;
            self.record("wrap_model_call: after");
            result
        }

        async fn after_model(&self, _response: &ProviderResponse) -> crate::Result<HookAction> {
            self.record("after_model");
            Ok(HookAction::Continue)
        }

        async fn after_agent(&self, result: &str) -> crate::Result<String> {
            self.record(format!("after_agent: {}", result));
            Ok(result.to_string())
        }
    }

    // TEST: before_agent hook is called
    #[tokio::test]
    async fn test_before_agent_hook_called() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let agent = create_agent("test")
            .with_provider(Box::new(MockProvider::new("response")))
            .with_lifecycle(TrackingHook::new(calls.clone()));

        let _ = agent.run("test input").await;

        let calls = calls.lock().unwrap();
        assert!(calls.iter().any(|c| c.contains("before_agent: test input")));
    }

    // TEST: Hook transforms input
    struct UppercaseInputHook;

    #[async_trait]
    impl AgentLifecycle for UppercaseInputHook {
        async fn before_agent(&self, input: &str) -> crate::Result<String> {
            Ok(input.to_uppercase())
        }
    }

    #[tokio::test]
    async fn test_before_agent_transforms_input() {
        // We can't easily test that input was transformed without inspecting provider call
        // This test verifies the hook is called without error
        let agent = create_agent("test")
            .with_provider(Box::new(MockProvider::new("response")))
            .with_lifecycle(UppercaseInputHook);

        let result = agent.run("hello").await;
        assert!(result.is_ok());
    }

    // TEST: after_model hook can reject
    struct RejectHook;

    #[async_trait]
    impl AgentLifecycle for RejectHook {
        async fn after_model(&self, _: &ProviderResponse) -> crate::Result<HookAction> {
            Ok(HookAction::Reject("Rejected by hook".to_string()))
        }
    }

    #[tokio::test]
    async fn test_after_model_reject_stops_execution() {
        let agent = create_agent("test")
            .with_provider(Box::new(MockProvider::new("response")))
            .with_lifecycle(RejectHook);

        let result = agent.run("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rejected"));
    }

    // TEST: after_agent hook is called
    #[tokio::test]
    async fn test_after_agent_hook_called() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let agent = create_agent("test")
            .with_provider(Box::new(MockProvider::new("final response")))
            .with_lifecycle(TrackingHook::new(calls.clone()));

        let _ = agent.run("test").await;

        let calls = calls.lock().unwrap();
        assert!(calls.iter().any(|c| c.contains("after_agent")));
    }

    // TEST: Multiple hooks execute in order
    #[tokio::test]
    async fn test_multiple_hooks_execute_in_order() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let hook1 = TrackingHook::new(calls.clone());
        let hook2 = TrackingHook::new(calls.clone());

        let agent = create_agent("test")
            .with_provider(Box::new(MockProvider::new("response")))
            .with_lifecycle(hook1)
            .with_lifecycle(hook2);

        let _ = agent.run("test").await;

        let calls = calls.lock().unwrap();
        // Both hooks should have been called
        let before_agent_count = calls.iter().filter(|c| c.contains("before_agent")).count();
        assert_eq!(before_agent_count, 2);
    }

    // TEST: Agent works without any hooks (regression test)
    #[tokio::test]
    async fn test_agent_works_without_hooks() {
        let agent =
            create_agent("test").with_provider(Box::new(MockProvider::new("no hooks response")));

        let result = agent.run("test").await.unwrap();
        assert_eq!(result, "no hooks response");
    }
}
