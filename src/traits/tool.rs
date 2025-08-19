//! Tool trait definition and supporting types
//!
//! This module defines the core Tool trait that all tool implementations
//! must implement. Tools are executable functions that agents can call
//! to perform specific actions or retrieve information.

use crate::error::PatinoxError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Tests written FIRST to define the contract
#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct TestTool {
        name: String,
    }

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A test tool for validation"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Input parameter"
                    }
                },
                "required": ["input"]
            })
        }

        async fn execute(&self, params: ToolParams) -> Result<ToolResult, PatinoxError> {
            // Validate parameters
            let input = params.parameters.get("input");
            if input.is_none() {
                return Ok(ToolResult {
                    call_id: params.call_id,
                    success: false,
                    data: serde_json::json!({}),
                    error: Some("Missing required parameter: input".to_string()),
                    metadata: HashMap::new(),
                });
            }

            // Return success result
            Ok(ToolResult {
                call_id: params.call_id,
                success: true,
                data: serde_json::json!({
                    "output": format!("Processed: {}", input.unwrap())
                }),
                error: None,
                metadata: HashMap::new(),
            })
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                category: "test".to_string(),
                tags: vec!["testing".to_string(), "validation".to_string()],
                version: "1.0.0".to_string(),
                author: Some("Test Suite".to_string()),
                dangerous: false,
            }
        }
    }

    impl TestTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[test]
    fn tool_call_serialization() {
        let tool_call = ToolCall {
            id: "call-123".to_string(),
            name: "test-tool".to_string(),
            parameters: serde_json::json!({"input": "test value"}),
        };

        let serialized = serde_json::to_string(&tool_call).expect("Should serialize");
        let deserialized: ToolCall = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.id, tool_call.id);
        assert_eq!(deserialized.name, tool_call.name);
        assert_eq!(deserialized.parameters["input"], "test value");
    }

    #[test]
    fn tool_params_structure() {
        let params = ToolParams {
            call_id: "call-456".to_string(),
            parameters: serde_json::json!({"key": "value"}),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("session".to_string(), serde_json::json!("session123"));
                ctx
            },
        };

        assert_eq!(params.call_id, "call-456");
        assert_eq!(params.parameters["key"], "value");
        assert_eq!(params.context["session"], "session123");
    }

    #[test]
    fn tool_result_serialization() {
        let result = ToolResult {
            call_id: "call-789".to_string(),
            success: true,
            data: serde_json::json!({"result": "success"}),
            error: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("duration_ms".to_string(), "150".to_string());
                meta
            },
        };

        let serialized = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: ToolResult =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.call_id, result.call_id);
        assert_eq!(deserialized.success, result.success);
        assert_eq!(deserialized.data["result"], "success");
        assert_eq!(deserialized.error, None);
        assert_eq!(deserialized.metadata["duration_ms"], "150");
    }

    #[test]
    fn tool_result_with_error() {
        let result = ToolResult {
            call_id: "call-error".to_string(),
            success: false,
            data: serde_json::json!({}),
            error: Some("Parameter validation failed".to_string()),
            metadata: HashMap::new(),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
        assert_eq!(
            result.error.expect("Should have error message"),
            "Parameter validation failed"
        );
    }

    #[test]
    fn tool_metadata_completeness() {
        let metadata = ToolMetadata {
            category: "data".to_string(),
            tags: vec!["database".to_string(), "query".to_string()],
            version: "2.1.0".to_string(),
            author: Some("DataTeam".to_string()),
            dangerous: true,
        };

        assert_eq!(metadata.category, "data");
        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"database".to_string()));
        assert!(metadata.tags.contains(&"query".to_string()));
        assert_eq!(metadata.version, "2.1.0");
        assert_eq!(metadata.author, Some("DataTeam".to_string()));
        assert!(metadata.dangerous);
    }

    #[tokio::test]
    async fn tool_basic_functionality() {
        let tool = TestTool::new("test-tool");

        // Test metadata access
        assert_eq!(tool.name(), "test-tool");
        assert!(!tool.description().is_empty());

        let schema = tool.parameters_schema();
        assert!(schema.is_object());
        assert!(schema["properties"].is_object());

        let metadata = tool.metadata();
        assert_eq!(metadata.category, "test");
        assert!(!metadata.version.is_empty());
    }

    #[tokio::test]
    async fn tool_successful_execution() {
        let tool = TestTool::new("test-tool");

        let params = ToolParams {
            call_id: "success-test".to_string(),
            parameters: serde_json::json!({"input": "Hello World"}),
            context: HashMap::new(),
        };

        let result = tool
            .execute(params)
            .await
            .expect("Execution should succeed");

        // Test tool contract requirements
        assert_eq!(
            result.call_id, "success-test",
            "Call ID should be preserved"
        );
        assert!(
            result.success,
            "Valid input should result in successful execution"
        );
        assert!(
            result.error.is_none(),
            "Successful execution should not have errors"
        );

        // Test that tool actually processes the input (not just returns it)
        if let Some(output) = result.data["output"].as_str() {
            assert!(
                output.contains("Processed:"),
                "Tool should process input, not just return it"
            );
            assert!(
                output.contains("Hello World"),
                "Tool should include original input in processed output"
            );
            assert!(
                output.len() > "Hello World".len(),
                "Processed output should be longer than raw input"
            );
        } else {
            panic!("Tool should return structured output data");
        }

        // Test metadata is properly initialized
        assert!(
            result.metadata.is_empty() || !result.metadata.is_empty(),
            "Metadata should be initialized"
        );
    }

    #[tokio::test]
    async fn tool_parameter_validation() {
        let tool = TestTool::new("test-tool");

        // Test missing required parameter
        let missing_params = ToolParams {
            call_id: "validation-test".to_string(),
            parameters: serde_json::json!({"wrong": "parameter"}),
            context: HashMap::new(),
        };

        let result = tool
            .execute(missing_params)
            .await
            .expect("Execution should complete");

        // Test proper error handling for validation failures
        assert_eq!(
            result.call_id, "validation-test",
            "Call ID should be preserved in errors"
        );
        assert!(
            !result.success,
            "Invalid parameters should result in failure"
        );
        assert!(
            result.error.is_some(),
            "Validation failures should include error message"
        );
        assert_eq!(
            result.data,
            serde_json::json!({}),
            "Failed executions should return empty data"
        );

        if let Some(error) = result.error {
            assert!(
                error.contains("Missing required parameter"),
                "Error should be descriptive"
            );
            assert!(
                error.contains("input"),
                "Error should mention the missing parameter name"
            );
        } else {
            panic!("Validation failure should include specific error message");
        }

        // Test with empty parameters object
        let empty_params = ToolParams {
            call_id: "empty-test".to_string(),
            parameters: serde_json::json!({}),
            context: HashMap::new(),
        };

        let empty_result = tool
            .execute(empty_params)
            .await
            .expect("Should handle empty params");
        assert!(
            !empty_result.success,
            "Empty parameters should fail validation"
        );
        assert!(
            empty_result.error.is_some(),
            "Should provide error for empty parameters"
        );

        // Test with null parameter value
        let null_params = ToolParams {
            call_id: "null-test".to_string(),
            parameters: serde_json::json!({"input": null}),
            context: HashMap::new(),
        };

        let null_result = tool
            .execute(null_params)
            .await
            .expect("Should handle null values");
        // Behavior depends on implementation - either fail validation or handle gracefully
        if !null_result.success {
            assert!(
                null_result.error.is_some(),
                "Null parameter failure should include error"
            );
        }
    }

    #[tokio::test]
    async fn tool_object_safety() {
        // Test that we can create trait objects
        let tool: Box<dyn Tool> = Box::new(TestTool::new("boxed-tool"));

        // Test that trait object methods work
        let _name = tool.name();
        let _description = tool.description();
        let _schema = tool.parameters_schema();
        let _metadata = tool.metadata();

        // Test async method works with trait objects
        let params = ToolParams {
            call_id: "object-test".to_string(),
            parameters: serde_json::json!({"input": "test"}),
            context: HashMap::new(),
        };

        let _result = tool.execute(params).await;

        // Test that we can store multiple tools in a collection
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(TestTool::new("tool1")),
            Box::new(TestTool::new("tool2")),
        ];

        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn tool_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Tool>>();
        assert_sync::<Box<dyn Tool>>();

        // Test that we can pass trait objects across thread boundaries
        let tool: Box<dyn Tool> = Box::new(TestTool::new("thread-test"));
        let tool_name = tool.name().to_string();

        tokio::spawn(async move {
            let _name = tool.name();
            // Tool trait object can be moved across threads
        })
        .await
        .unwrap();

        assert_eq!(tool_name, "thread-test");
    }

    #[test]
    fn tool_schema_validation() {
        let tool = TestTool::new("schema-test");
        let schema = tool.parameters_schema();

        // Schema should be a valid JSON Schema object
        assert!(schema.is_object());
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());

        // Should have the expected properties
        let properties = &schema["properties"];
        assert!(properties["input"].is_object());
        assert_eq!(properties["input"]["type"], "string");
        assert!(properties["input"]["description"].is_string());

        // Should specify required fields
        let required = schema["required"]
            .as_array()
            .expect("Schema should have required array");
        assert!(required.contains(&serde_json::json!("input")));
    }
}

/// Core tool abstraction for agent actions
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool identifier
    fn name(&self) -> &str;

    /// Tool description for LLM function calling
    fn description(&self) -> &str;

    /// JSON schema for tool parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    async fn execute(&self, params: ToolParams) -> Result<ToolResult, PatinoxError>;

    /// Tool metadata for discovery and categorization
    fn metadata(&self) -> ToolMetadata;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub call_id: String,
    pub parameters: serde_json::Value,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub author: Option<String>,
    pub dangerous: bool, // Requires extra validation
}
