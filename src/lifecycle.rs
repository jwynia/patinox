//! Agent lifecycle hooks for middleware and intervention points
//!
//! The lifecycle system provides 6 hook points where middleware can intercept,
//! modify, or observe agent execution:
//!
//! - `before_agent`: Transform input before processing
//! - `before_model`: Modify messages before LLM call
//! - `wrap_model_call`: Wrap LLM calls (retry, fallback, logging)
//! - `after_model`: Inspect/modify response, HITL approval
//! - `wrap_tool_call`: Wrap tool execution (retry, logging)
//! - `after_agent`: Transform final result
//!
//! # Example
//! ```ignore
//! struct LoggingHook;
//!
//! #[async_trait]
//! impl AgentLifecycle for LoggingHook {
//!     async fn before_agent(&self, input: &str) -> Result<String> {
//!         println!("Input: {}", input);
//!         Ok(input.to_string())
//!     }
//! }
//!
//! let agent = create_agent("my-agent")
//!     .with_lifecycle(LoggingHook);
//! ```

use crate::provider::{Message, ProviderResponse, ProviderResult};
use crate::tool::ToolResult;
use async_trait::async_trait;
use std::future::Future;

/// Action to take after a lifecycle hook
///
/// Used by `after_model` hook to control agent execution flow.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Continue processing normally
    Continue,

    /// Explicit approval (for HITL workflows)
    Approve,

    /// Reject with error message
    Reject(String),

    /// Modify the provider response
    Modify(ProviderResponse),
}

/// Agent lifecycle hooks for middleware and intervention points
///
/// All methods have default implementations that pass through without modification.
/// Implement only the hooks you need for your middleware.
///
/// # Hook Execution Order
///
/// 1. `before_agent` - Called once before agent starts processing
/// 2. `before_model` - Called before each LLM call (in tool-calling loop)
/// 3. `wrap_model_call` - Wraps the LLM call itself
/// 4. `after_model` - Called after each LLM response
/// 5. `wrap_tool_call` - Wraps each tool execution
/// 6. `after_agent` - Called once before returning final result
///
/// # Example: Logging Hook
///
/// ```ignore
/// use patinox::*;
///
/// struct LoggingHook;
///
/// #[async_trait]
/// impl AgentLifecycle for LoggingHook {
///     async fn before_agent(&self, input: &str) -> Result<String> {
///         println!("[AGENT] Starting with input: {}", input);
///         Ok(input.to_string())
///     }
///
///     async fn after_agent(&self, result: &str) -> Result<String> {
///         println!("[AGENT] Completed with result: {}", result);
///         Ok(result.to_string())
///     }
/// }
/// ```
#[async_trait]
pub trait AgentLifecycle: Send + Sync {
    /// Called before agent starts processing input
    ///
    /// Use this for:
    /// - Input validation and sanitization
    /// - Rate limiting checks
    /// - Loading context or session state
    /// - Transforming input format
    ///
    /// # Arguments
    /// * `input` - The user input string
    ///
    /// # Returns
    /// Transformed input (or same input if no changes)
    async fn before_agent(&self, input: &str) -> crate::Result<String> {
        Ok(input.to_string())
    }

    /// Called before sending messages to LLM
    ///
    /// Use this for:
    /// - Context window management (trimming old messages)
    /// - Message compression or summarization
    /// - Prompt injection or modification
    /// - Adding system messages
    ///
    /// # Arguments
    /// * `messages` - Current conversation messages
    ///
    /// # Returns
    /// Transformed message list
    async fn before_model(&self, messages: Vec<Message>) -> crate::Result<Vec<Message>> {
        Ok(messages)
    }

    /// Wraps the model call (for retry, fallback, etc.)
    ///
    /// Use this for:
    /// - Retry logic with exponential backoff
    /// - Fallback to alternative providers
    /// - Caching responses
    /// - Telemetry and timing
    ///
    /// # Arguments
    /// * `f` - The actual model call as a boxed future
    ///
    /// # Returns
    /// Provider response (or error)
    async fn wrap_model_call(
        &self,
        f: std::pin::Pin<Box<dyn Future<Output = ProviderResult<ProviderResponse>> + Send>>,
    ) -> ProviderResult<ProviderResponse> {
        f.await
    }

    /// Called after model responds, before tool execution
    ///
    /// Use this for:
    /// - Human-in-the-loop approval workflows
    /// - Safety validation and content filtering
    /// - Response modification or enhancement
    /// - Deciding to reject or approve continuation
    ///
    /// # Arguments
    /// * `response` - The provider's response
    ///
    /// # Returns
    /// Action to take (Continue, Approve, Reject, Modify)
    async fn after_model(&self, _response: &ProviderResponse) -> crate::Result<HookAction> {
        Ok(HookAction::Continue)
    }

    /// Wraps each tool call (for retry, logging, etc.)
    ///
    /// Use this for:
    /// - Tool-specific retry logic
    /// - Permission checks before execution
    /// - Audit logging of tool usage
    /// - Dry-run mode (intercept without executing)
    ///
    /// # Arguments
    /// * `name` - Name of the tool being called
    /// * `f` - The actual tool execution as a boxed future
    ///
    /// # Returns
    /// Tool result string (or error)
    async fn wrap_tool_call(
        &self,
        _name: &str,
        f: std::pin::Pin<Box<dyn Future<Output = ToolResult> + Send>>,
    ) -> ToolResult {
        f.await
    }

    /// Called after agent completes execution
    ///
    /// Use this for:
    /// - Result formatting or post-processing
    /// - Saving results to storage
    /// - Sending notifications
    /// - Collecting metrics
    ///
    /// # Arguments
    /// * `result` - The final agent result
    ///
    /// # Returns
    /// Transformed result (or same result if no changes)
    async fn after_agent(&self, result: &str) -> crate::Result<String> {
        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Message, ProviderResponse};

    // Default hook implementation for testing
    struct DefaultHook;

    #[async_trait]
    impl AgentLifecycle for DefaultHook {}

    // TEST 1: before_agent default implementation passes through
    #[tokio::test]
    async fn test_before_agent_default_passthrough() {
        let hook = DefaultHook;
        let input = "test input";
        let result = hook.before_agent(input).await.unwrap();
        assert_eq!(result, input);
    }

    // TEST 2: before_model default implementation passes through
    #[tokio::test]
    async fn test_before_model_default_passthrough() {
        let hook = DefaultHook;
        let messages = vec![
            Message::system("system prompt"),
            Message::user("user message"),
        ];
        let result = hook.before_model(messages.clone()).await.unwrap();
        assert_eq!(result.len(), messages.len());
        assert_eq!(result[0].role, "system");
        assert_eq!(result[1].role, "user");
    }

    // TEST 3: wrap_model_call default implementation calls function
    #[tokio::test]
    async fn test_wrap_model_call_default_executes() {
        let hook = DefaultHook;
        let response = ProviderResponse::Text("test response".to_string());
        let future = Box::pin(async move { Ok(response) });
        let result = hook.wrap_model_call(future).await.unwrap();

        match result {
            ProviderResponse::Text(text) => assert_eq!(text, "test response"),
            _ => panic!("Expected Text response"),
        }
    }

    // TEST 4: after_model default implementation returns Continue
    #[tokio::test]
    async fn test_after_model_default_continues() {
        let hook = DefaultHook;
        let response = ProviderResponse::Text("test".to_string());
        let result = hook.after_model(&response).await.unwrap();

        match result {
            HookAction::Continue => {} // Expected
            _ => panic!("Expected HookAction::Continue"),
        }
    }

    // TEST 5: wrap_tool_call default implementation calls function
    #[tokio::test]
    async fn test_wrap_tool_call_default_executes() {
        let hook = DefaultHook;
        let future = Box::pin(async { Ok("tool result".to_string()) });
        let result = hook.wrap_tool_call("test_tool", future).await.unwrap();

        assert_eq!(result, "tool result");
    }

    // TEST 6: after_agent default implementation passes through
    #[tokio::test]
    async fn test_after_agent_default_passthrough() {
        let hook = DefaultHook;
        let result_str = "final result";
        let result = hook.after_agent(result_str).await.unwrap();
        assert_eq!(result, result_str);
    }

    // TEST 7: Custom hook can transform input
    struct UppercaseHook;

    #[async_trait]
    impl AgentLifecycle for UppercaseHook {
        async fn before_agent(&self, input: &str) -> crate::Result<String> {
            Ok(input.to_uppercase())
        }
    }

    #[tokio::test]
    async fn test_custom_hook_transforms_input() {
        let hook = UppercaseHook;
        let result = hook.before_agent("hello").await.unwrap();
        assert_eq!(result, "HELLO");
    }

    // TEST 8: HookAction::Reject variant
    #[test]
    fn test_hook_action_reject() {
        let action = HookAction::Reject("test error".to_string());
        match action {
            HookAction::Reject(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Reject variant"),
        }
    }

    // TEST 9: HookAction::Modify variant
    #[test]
    fn test_hook_action_modify() {
        let response = ProviderResponse::Text("modified".to_string());
        let action = HookAction::Modify(response.clone());
        match action {
            HookAction::Modify(r) => match r {
                ProviderResponse::Text(text) => assert_eq!(text, "modified"),
                _ => panic!("Expected Text response"),
            },
            _ => panic!("Expected Modify variant"),
        }
    }
}
