//! Tool Context Helper - Eliminate Closure Capture Boilerplate
//!
//! This module provides extension methods that automatically capture context
//! variables for tools, eliminating the manual clone + move pattern.
//!
//! # Problem
//!
//! Without this plugin, tools that need context require manual boilerplate:
//! ```ignore
//! .tool_fn("read_file", "Read file", {
//!     let path = file_path.clone();  // ❌ Manual clone
//!     move |_args| read_file_tool(&path)  // ❌ Manual move
//! })
//! ```
//!
//! # Solution
//!
//! The `ToolContextExt` trait provides methods that handle cloning automatically:
//! ```ignore
//! use patinox::plugin::tool_context::ToolContextExt;
//!
//! .tool_fn_with("read_file", "Read file", &file_path,
//!     |path, _args| read_file_tool(path))  // ✅ Zero boilerplate
//! ```
//!
//! # Performance
//!
//! Zero runtime overhead - compiles to exactly the same code as manual clone + move.

use crate::agent::Agent;
use crate::tool::ToolResult;

/// Extension trait for Agent to support automatic context capture
///
/// This trait provides convenient methods for adding tools that need access
/// to external context (file paths, config, state, etc.) without requiring
/// manual clone + move boilerplate.
///
/// # Examples
///
/// ## Single Context Variable
/// ```ignore
/// use patinox::prelude::*;
/// use patinox::plugin::tool_context::ToolContextExt;
///
/// let file_path = String::from("data.txt");
///
/// let agent = create_agent("demo")
///     .tool_fn_with("read_file", "Read file contents", &file_path,
///         |path, _args| {
///             // path is &String, automatically captured
///             Ok(format!("Reading {}", path))
///         });
/// ```
///
/// ## Two Context Variables
/// ```ignore
/// let file_path = String::from("data.txt");
/// let config = String::from("config.json");
///
/// let agent = create_agent("demo")
///     .tool_fn_with2("process", "Process with config", &file_path, &config,
///         |path, cfg, _args| {
///             Ok(format!("Processing {} with {}", path, cfg))
///         });
/// ```
pub trait ToolContextExt {
    /// Add a tool with one captured context variable
    ///
    /// The context is cloned once and passed to the handler closure.
    /// This eliminates the need for manual `clone()` + `move` boilerplate.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Context type (must be `Clone + Send + Sync + 'static`)
    /// - `F`: Handler function `Fn(&T, String) -> ToolResult`
    ///
    /// # Arguments
    ///
    /// - `name`: Tool name
    /// - `desc`: Tool description
    /// - `context`: Context to capture (passed by reference)
    /// - `handler`: Tool handler receiving context and arguments
    ///
    /// # Example
    /// ```ignore
    /// let file_path = String::from("test.txt");
    ///
    /// agent.tool_fn_with("read", "Read file", &file_path, |path, args| {
    ///     // path is &String
    ///     Ok(format!("Reading {}", path))
    /// })
    /// ```
    fn tool_fn_with<T, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        context: &T,
        handler: F,
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T, String) -> ToolResult + Send + Sync + 'static;

    /// Add a tool with two captured context variables
    ///
    /// Both contexts are cloned once and passed to the handler closure.
    ///
    /// # Type Parameters
    ///
    /// - `T1`, `T2`: Context types (must be `Clone + Send + Sync + 'static`)
    /// - `F`: Handler function `Fn(&T1, &T2, String) -> ToolResult`
    ///
    /// # Arguments
    ///
    /// - `name`: Tool name
    /// - `desc`: Tool description
    /// - `ctx1`, `ctx2`: Contexts to capture
    /// - `handler`: Tool handler receiving both contexts and arguments
    ///
    /// # Example
    /// ```ignore
    /// let path = String::from("data.txt");
    /// let config = String::from("config.json");
    ///
    /// agent.tool_fn_with2("process", "Process file", &path, &config,
    ///     |p, c, args| {
    ///         Ok(format!("Processing {} with config {}", p, c))
    ///     })
    /// ```
    fn tool_fn_with2<T1, T2, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        ctx1: &T1,
        ctx2: &T2,
        handler: F,
    ) -> Self
    where
        T1: Clone + Send + Sync + 'static,
        T2: Clone + Send + Sync + 'static,
        F: Fn(&T1, &T2, String) -> ToolResult + Send + Sync + 'static;
}

impl ToolContextExt for Agent {
    fn tool_fn_with<T, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        context: &T,
        handler: F,
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T, String) -> ToolResult + Send + Sync + 'static,
    {
        // Clone context once here (instead of requiring user to do it)
        let ctx = context.clone();
        // Move the cloned context into the closure
        self.tool_fn(name, desc, move |args| handler(&ctx, args))
    }

    fn tool_fn_with2<T1, T2, F>(
        self,
        name: impl Into<String>,
        desc: impl Into<String>,
        ctx1: &T1,
        ctx2: &T2,
        handler: F,
    ) -> Self
    where
        T1: Clone + Send + Sync + 'static,
        T2: Clone + Send + Sync + 'static,
        F: Fn(&T1, &T2, String) -> ToolResult + Send + Sync + 'static,
    {
        // Clone both contexts once
        let c1 = ctx1.clone();
        let c2 = ctx2.clone();
        // Move both into the closure
        self.tool_fn(name, desc, move |args| handler(&c1, &c2, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_agent;
    use serde_json::json;

    #[test]
    fn test_tool_fn_with_single_context() {
        let file_path = String::from("test.txt");

        let agent = create_agent("test")
            .tool_fn_with("read_file", "Read file", &file_path, |path, _args| {
                Ok(format!("Reading {}", path))
            });

        // Verify tool was added
        assert!(agent.tools.contains_key("read_file"));
    }

    #[test]
    fn test_tool_fn_with_executes_correctly() {
        let file_path = String::from("data.txt");

        let agent = create_agent("test")
            .tool_fn_with("read_file", "Read file", &file_path, |path, _args| {
                Ok(format!("Reading {}", path))
            });

        // Execute the tool directly
        let tool = agent.tools.get("read_file").unwrap();
        let result = tool.execute(json!({})).unwrap();
        assert_eq!(result, "Reading data.txt");
    }

    #[test]
    fn test_tool_fn_with2_dual_context() {
        let file_path = String::from("test.txt");
        let config = String::from("config.json");

        let agent = create_agent("test").tool_fn_with2(
            "process",
            "Process file",
            &file_path,
            &config,
            |path, cfg, _args| Ok(format!("Processing {} with {}", path, cfg)),
        );

        // Verify tool was added
        assert!(agent.tools.contains_key("process"));
    }

    #[test]
    fn test_tool_fn_with2_executes_correctly() {
        let file_path = String::from("data.txt");
        let config = String::from("settings.json");

        let agent = create_agent("test").tool_fn_with2(
            "process",
            "Process file",
            &file_path,
            &config,
            |path, cfg, _args| Ok(format!("Processing {} with {}", path, cfg)),
        );

        let tool = agent.tools.get("process").unwrap();
        let result = tool.execute(json!({})).unwrap();
        assert_eq!(result, "Processing data.txt with settings.json");
    }

    #[test]
    fn test_multiple_tools_with_same_context() {
        let file_path = String::from("shared.txt");

        let agent = create_agent("test")
            .tool_fn_with("read", "Read file", &file_path, |path, _args| {
                Ok(format!("Reading {}", path))
            })
            .tool_fn_with("info", "File info", &file_path, |path, _args| {
                Ok(format!("Info for {}", path))
            });

        assert!(agent.tools.contains_key("read"));
        assert!(agent.tools.contains_key("info"));

        // Both tools should have access to the same context value
        let read_result = agent
            .tools
            .get("read")
            .unwrap()
            .execute(json!({}))
            .unwrap();
        let info_result = agent
            .tools
            .get("info")
            .unwrap()
            .execute(json!({}))
            .unwrap();

        assert!(read_result.contains("shared.txt"));
        assert!(info_result.contains("shared.txt"));
    }

    #[test]
    fn test_tool_fn_with_maintains_builder_pattern() {
        let file_path = String::from("test.txt");

        // Verify builder pattern works
        let agent = create_agent("test")
            .tool_fn_with("tool1", "First tool", &file_path, |path, _| {
                Ok(format!("Tool 1: {}", path))
            })
            .tool_fn_with("tool2", "Second tool", &file_path, |path, _| {
                Ok(format!("Tool 2: {}", path))
            });

        assert_eq!(agent.tools.len(), 2);
        assert!(agent.tools.contains_key("tool1"));
        assert!(agent.tools.contains_key("tool2"));
    }
}
