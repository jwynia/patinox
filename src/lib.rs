//! Patinox - Minimal to Sophisticated AI Agents in Rust
//!
//! A layered agent framework that starts simple and grows with your needs.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use patinox::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = create_agent("hello")
//!         .tool_fn("greet", "Say hello", |name: String| {
//!             Ok(format!("Hello, {}!", name))
//!         })?;
//!
//!     agent.run_cli()
//! }
//! ```

pub mod agent;
pub mod provider;
pub mod tool;
pub mod cli;

pub use agent::{Agent, AgentConfig, create_agent};
pub use provider::{Provider, LLMProvider};
pub use tool::{Tool, FnTool};
pub use cli::run_cli;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{Agent, AgentConfig, create_agent, Provider, Tool, FnTool, run_cli};
}

/// Re-export commonly used types
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
