//! # Patinox - Self-Improving AI Agent Framework
//!
//! Patinox is a production-ready AI agent framework built in Rust that emphasizes
//! safety, observability, and continuous improvement through automated monitoring
//! and meta-layer analysis.
//!
//! ## Core Features
//!
//! - **Type-Safe Agent Abstractions**: Compile-time guarantees for agent behavior
//! - **Validation Pipeline**: Synchronous safety checks using Tower middleware
//! - **Monitoring Layer**: Asynchronous telemetry collection and analysis
//! - **Meta-Layer Evolution**: Git-based self-improvement system
//! - **Zero-Cost Abstractions**: Maximum performance with compile-time optimization
//!
//! ## Architecture
//!
//! Patinox implements the MAPE-K (Monitor-Analyze-Plan-Execute over shared Knowledge)
//! pattern for self-adaptive agent behavior:
//!
//! ```text
//! Monitor → Analyze → Plan → Execute
//!     ↑                        ↓
//!     ←── Shared Knowledge ←────
//! ```
//!
//! ## Getting Started
//!
//! ```rust
//! use patinox::prelude::*;
//!
//! // Example usage will be added as the framework develops
//! ```
//!
//! ## Development Status
//!
//! 🚧 This framework is under active development. Core features are being
//! implemented following a test-driven, utility-first approach.
//!
//! Current implementation phase: **Foundation** - establishing core error types,
//! traits, and utility functions.

// Re-export core types when they become available
pub mod prelude {
    //! Common imports for working with Patinox
    //!
    //! This module re-exports the most commonly used types and traits
    //! for convenient importing with `use patinox::prelude::*;`

    // Re-export library metadata
    pub use crate::{NAME, VERSION};

    // Core types will be re-exported here as they're implemented
}

// Library version and metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_metadata() {
        // Test that version is a valid semver-like format (has dots)
        assert!(
            VERSION.contains('.'),
            "Version should be a semver format: {}",
            VERSION
        );
        assert_eq!(NAME, "patinox", "Package name should be patinox");
    }

    #[test]
    fn test_prelude_module_exists() {
        // This test ensures the prelude module is accessible
        // As we add items to prelude, we can test they're accessible here

        // For now, just verify the module exists by accessing it
        let _prelude = crate::prelude::VERSION;
        // When we add actual exports to prelude, we'll test them here
    }
}
