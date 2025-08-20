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

// Core modules
pub mod error;
pub mod traits;

// Type safety infrastructure
pub mod builder;
pub mod typestate;

// Memory management utilities
pub mod memory;

// Re-export core types when they become available
pub mod prelude {
    //! Common imports for working with Patinox
    //!
    //! This module re-exports the most commonly used types and traits
    //! for convenient importing with `use patinox::prelude::*;`

    // Re-export library metadata
    pub use crate::{NAME, VERSION};

    // Re-export core error types for convenient access
    pub use crate::error::{
        ConfigurationError, ExecutionError, NetworkError, PatinoxError, RecoveryStrategy,
        ValidationError,
    };

    // Re-export core trait interfaces
    pub use crate::traits::{
        Agent, AgentBuilder, AgentConfig, AgentRequest, AgentResponse, AgentState,
        ExecutionSummary, HealthStatus, Monitor, MonitorConfig, MonitorEvent, MonitorEventType,
        MonitorQuery, Tool, ToolCall, ToolMetadata, ToolParams, ToolResult, Usage,
        ValidationContent, ValidationModifications, ValidationRequest, ValidationResponse,
        ValidationStage, Validator, ValidatorConfig,
    };

    // Re-export type safety infrastructure
    pub use crate::builder::{
        BuilderState, CompleteBuilder, ConfigBuilder, EmptyBuilder, PartialBuilder,
    };
    pub use crate::typestate::{
        AgentWrapper, Created, Running, Started, StateMarker, Stopped, TypeSafeAgentBuilder,
    };
    
    // Re-export memory management utilities
    pub use crate::memory::{
        AsyncResourceGuard, CleanupError, CleanupPriority, ResourceId,
        ResourceInfo, ResourceRegistry, 
    };
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

        // Version should follow semver pattern
        let version_parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            version_parts.len() >= 2,
            "Version should have at least major.minor format: {}",
            VERSION
        );

        // Each version part should be numeric
        for part in version_parts.iter().take(3) {
            assert!(
                part.chars().all(|c| c.is_ascii_digit()),
                "Version part '{}' should be numeric in version: {}",
                part,
                VERSION
            );
        }
    }

    #[test]
    fn test_prelude_module_exports() {
        // Verify prelude re-exports are accessible
        let _version = crate::prelude::VERSION;
        let _name = crate::prelude::NAME;

        // Verify error types are accessible through prelude
        use crate::prelude::*;

        // Test that we can create errors through prelude imports
        let _validation_error = ValidationError::InvalidInput("test".to_string());
        let _patinox_error = PatinoxError::Validation(_validation_error);
        let _recovery_strategy = RecoveryStrategy::Retry;

        // Verify types implement expected traits
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<PatinoxError>();
        assert_sync::<PatinoxError>();
        assert_send::<RecoveryStrategy>();
        assert_sync::<RecoveryStrategy>();
    }

    #[test]
    fn test_prelude_completeness() {
        // Verify all major error types are re-exported in prelude
        use crate::prelude::*;

        // Should be able to construct all main error variants
        let _config_error = ConfigurationError::InvalidFormat("test".to_string());
        let _exec_error = ExecutionError::ResourceExhausted("memory".to_string());
        let _net_error = NetworkError::Timeout("api timeout".to_string());
        let _val_error = ValidationError::RateLimited;

        // All recovery strategies should be available
        let strategies = [
            RecoveryStrategy::Retry,
            RecoveryStrategy::Fallback,
            RecoveryStrategy::CircuitBreak,
            RecoveryStrategy::Fail,
        ];

        // Should be able to use all strategies
        for strategy in strategies {
            assert!(matches!(
                strategy,
                RecoveryStrategy::Retry
                    | RecoveryStrategy::Fallback
                    | RecoveryStrategy::CircuitBreak
                    | RecoveryStrategy::Fail
            ));
        }
    }
}
