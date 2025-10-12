//! Tool system for Patinox agents
//!
//! Tools are functions that agents can call. The minimal implementation
//! supports simple string-based tools with easy integration.

use serde_json::Value;
use std::sync::Arc;

/// Result type for tool execution
pub type ToolResult = Result<String, Box<dyn std::error::Error + Send + Sync>>;

/// Tool trait - anything that can be executed by an agent
pub trait Tool: Send + Sync {
    /// Name of the tool (used by LLM to identify it)
    fn name(&self) -> &str;

    /// Description of what the tool does (helps LLM decide when to use it)
    fn description(&self) -> &str;

    /// Execute the tool with JSON arguments
    fn execute(&self, args: Value) -> ToolResult;
}

/// Function-based tool - wraps a closure as a Tool
pub struct FnTool {
    name: String,
    description: String,
    handler: Arc<dyn Fn(Value) -> ToolResult + Send + Sync>,
}

impl FnTool {
    /// Create a new function-based tool
    pub fn new<F>(name: impl Into<String>, description: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Value) -> ToolResult + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            handler: Arc::new(handler),
        }
    }

    /// Helper to create a tool from a function that takes a String
    pub fn from_string_fn<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(String) -> ToolResult + Send + Sync + 'static,
    {
        let name_str = name.into();
        let desc_str = description.into();

        Self::new(name_str, desc_str, move |args: Value| {
            // Extract string argument (either direct string or "input" field)
            let input = if let Some(s) = args.as_str() {
                s.to_string()
            } else if let Some(obj) = args.as_object() {
                obj.get("input")
                    .or_else(|| obj.get("text"))
                    .or_else(|| obj.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            } else {
                String::new()
            };

            handler(input)
        })
    }
}

impl Tool for FnTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn execute(&self, args: Value) -> ToolResult {
        (self.handler)(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fn_tool_creation() {
        let tool = FnTool::new("test", "A test tool", |_| Ok("result".to_string()));
        assert_eq!(tool.name(), "test");
        assert_eq!(tool.description(), "A test tool");
    }

    #[test]
    fn test_fn_tool_execution() {
        let tool = FnTool::new("echo", "Echo input", |args| {
            Ok(args.to_string())
        });

        let result = tool.execute(json!({"input": "hello"})).unwrap();
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_string_fn_tool() {
        let tool = FnTool::from_string_fn("uppercase", "Convert to uppercase", |input| {
            Ok(input.to_uppercase())
        });

        let result = tool.execute(json!({"input": "hello"})).unwrap();
        assert_eq!(result, "HELLO");
    }
}
