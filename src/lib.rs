//! Patinox - Minimal to Sophisticated AI Agents in Rust
//!
//! A layered agent framework that starts simple and grows with your needs.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use patinox::*;
//!
//! fn main() -> patinox::Result<()> {
//!     let agent = create_agent("hello")
//!         .tool_fn("greet", "Say hello", |name: String| {
//!             Ok(format!("Hello, {}!", name))
//!         });
//!
//!     agent.run_cli()
//! }
//! ```

pub mod agent;
pub mod cli;
pub mod plugin;
pub mod provider;
pub mod tool;

pub use agent::{create_agent, Agent, AgentConfig};
pub use cli::run_cli;
pub use provider::{LLMProvider, OpenAIProvider, Provider};
pub use tool::{FnTool, Tool};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::tool::ToolResult;
    pub use crate::{create_agent, run_cli, Agent, AgentConfig, FnTool, Provider, Tool};
}

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
