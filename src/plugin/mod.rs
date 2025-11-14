//! Plugin system for extending agent functionality
//!
//! The plugin system allows optional enhancements to agent capabilities
//! without adding complexity to the core framework. Each plugin is opt-in
//! and designed to solve specific pain points discovered through real usage.
//!
//! ## V2 Plugin Philosophy
//!
//! Plugins follow the V2 principle: **pain-driven development**
//! - Plugins are only created after pain is validated across multiple agents
//! - Each plugin solves a specific, measured problem
//! - Plugins are optional - core framework works without them
//! - Zero-cost abstractions - plugins compile away to manual code
//!
//! ## Available Plugins
//!
//! ### Tool Context Helper (V2-PLUGIN-001-B)
//! **Status**: Implemented ✅
//! **Pain Score**: 30/30 (Critical)
//! **Problem**: Manual clone + move boilerplate for context-aware tools
//! **Solution**: Extension methods like `.tool_fn_with()` that capture context automatically
//!
//! See: [`tool_context`] module for implementation and usage examples

use crate::agent::Agent;

// Plugin modules
pub mod tool_context; // V2-PLUGIN-001-B (Tool Context Helper)
                      // pub mod cli;           // V2-PLUGIN-002 (Future)
                      // pub mod discovery;     // V2-PLUGIN-003 (Future)

// Re-export for convenience
pub use tool_context::ToolContextExt;

/// Plugin trait for extending agents
///
/// Plugins implement this trait to provide optional functionality
/// that can be applied to agents during construction.
///
/// ## Design Principles
/// - **Opt-in**: Plugins are not applied automatically
/// - **Zero-cost**: Should compile to the same code as manual implementation
/// - **Type-safe**: Use Rust's type system for safety, not runtime checks
/// - **Composable**: Multiple plugins can be used together
pub trait AgentPlugin: Send + Sync {
    /// Plugin name (for debugging and documentation)
    fn name(&self) -> &str;

    /// Apply plugin transformations to an agent
    ///
    /// This is called during agent construction to add plugin-specific
    /// functionality. Implementations should use the builder pattern
    /// to maintain fluent API.
    fn apply(&self, agent: Agent) -> Agent;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_agent, Agent};

    /// Test plugin that adds a suffix to the agent name
    struct NameSuffixPlugin {
        suffix: String,
    }

    impl NameSuffixPlugin {
        fn new(suffix: impl Into<String>) -> Self {
            Self {
                suffix: suffix.into(),
            }
        }
    }

    impl AgentPlugin for NameSuffixPlugin {
        fn name(&self) -> &str {
            "NameSuffixPlugin"
        }

        fn apply(&self, mut agent: Agent) -> Agent {
            agent.config.name = format!("{}-{}", agent.config.name, self.suffix);
            agent
        }
    }

    /// Test plugin that sets a description
    struct DescriptionPlugin {
        description: String,
    }

    impl DescriptionPlugin {
        fn new(description: impl Into<String>) -> Self {
            Self {
                description: description.into(),
            }
        }
    }

    impl AgentPlugin for DescriptionPlugin {
        fn name(&self) -> &str {
            "DescriptionPlugin"
        }

        fn apply(&self, mut agent: Agent) -> Agent {
            agent.config.description = Some(self.description.clone());
            agent
        }
    }

    #[test]
    fn test_plugin_applies_to_agent() {
        let agent = create_agent("test").with_plugin(NameSuffixPlugin::new("v1"));

        assert_eq!(agent.config.name, "test-v1");
    }

    #[test]
    fn test_multiple_plugins_compose() {
        let agent = create_agent("test")
            .with_plugin(NameSuffixPlugin::new("v1"))
            .with_plugin(DescriptionPlugin::new("A test agent"));

        assert_eq!(agent.config.name, "test-v1");
        assert_eq!(agent.config.description, Some("A test agent".to_string()));
    }

    #[test]
    fn test_plugin_name() {
        let plugin = NameSuffixPlugin::new("test");
        assert_eq!(plugin.name(), "NameSuffixPlugin");
    }

    #[test]
    fn test_plugins_execute_in_order() {
        // Apply two suffix plugins - they should execute in order
        let agent = create_agent("base")
            .with_plugin(NameSuffixPlugin::new("first"))
            .with_plugin(NameSuffixPlugin::new("second"));

        assert_eq!(agent.config.name, "base-first-second");
    }
}
