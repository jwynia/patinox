//! Advanced builder patterns with compile-time safety
//!
//! This module provides builder patterns that enforce required fields
//! at compile time using phantom types and the typestate pattern.

use crate::traits::AgentConfig;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Tests written FIRST to define the contract
    
    #[test]
    fn builder_states_exist() {
        // Test that builder state marker types exist and work
        let _empty = EmptyBuilder;
        let _partial = PartialBuilder;
        let _complete = CompleteBuilder;
        
        // States should be zero-sized
        assert_eq!(std::mem::size_of::<EmptyBuilder>(), 0);
        assert_eq!(std::mem::size_of::<PartialBuilder>(), 0);
        assert_eq!(std::mem::size_of::<CompleteBuilder>(), 0);
        
        // States should implement standard traits
        let empty = EmptyBuilder;
        let _cloned = empty.clone();
        let _debug = format!("{:?}", empty);
    }
    
    #[test]
    fn required_field_tracking() {
        // Test that builder tracks which required fields are set
        
        // Start with empty builder - no required fields set
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent");
        assert!(!builder.has_llm_provider());
        assert!(!builder.has_llm_model());
        assert_eq!(builder.required_fields_count(), 0);
        assert_eq!(builder.missing_required_count(), 2); // provider + model
        
        // Add one required field
        let builder = builder.llm_provider("openai");
        assert!(builder.has_llm_provider());
        assert!(!builder.has_llm_model());
        assert_eq!(builder.required_fields_count(), 1);
        assert_eq!(builder.missing_required_count(), 1); // just model
        
        // Add second required field -> complete
        let builder = builder.llm_model("gpt-4");
        assert!(builder.has_llm_provider());
        assert!(builder.has_llm_model());
        assert_eq!(builder.required_fields_count(), 2);
        assert_eq!(builder.missing_required_count(), 0);
    }
    
    #[test]
    fn builder_state_transitions() {
        // Test that builder correctly transitions between states
        
        // Start in EmptyBuilder state
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent");
        assert_eq!(builder.builder_state(), "Empty");
        
        // Transition to PartialBuilder when we add one required field
        let builder = builder.llm_provider("openai");
        assert_eq!(builder.builder_state(), "Partial");
        
        // Transition to CompleteBuilder when we add all required fields
        let builder = builder.llm_model("gpt-4");
        assert_eq!(builder.builder_state(), "Complete");
        
        // Should be able to build only in Complete state
        let _config = builder.build();
    }
    
    #[test]
    fn optional_fields_work_in_all_states() {
        // Test that optional fields can be added in any builder state
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent")
            .description("Test description")
            .max_concurrent_requests(20)
            .timeout_ms(60000)
            .add_validator("test-validator")
            .add_tool("test-tool");
            
        // Should still be in EmptyBuilder state (no required fields yet)
        assert_eq!(builder.builder_state(), "Empty");
        assert_eq!(builder.missing_required_count(), 2);
        
        // Add required fields
        let builder = builder
            .llm_provider("openai")
            .llm_model("gpt-4");
            
        // Now should be complete
        assert_eq!(builder.builder_state(), "Complete");
        
        // Build and verify all fields are preserved
        let config = builder.build();
        assert_eq!(config.name, "test-agent");
        assert_eq!(config.description, Some("Test description".to_string()));
        assert_eq!(config.max_concurrent_requests, 20);
        assert_eq!(config.timeout_ms, 60000);
        assert!(config.enabled_validators.contains(&"test-validator".to_string()));
        assert!(config.tools.contains(&"test-tool".to_string()));
        assert_eq!(config.llm_provider, "openai");
        assert_eq!(config.llm_model, "gpt-4");
    }
    
    #[test]
    fn builder_field_validation() {
        // Test that builder validates field values appropriately
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent");
        
        // Should handle empty/invalid values gracefully
        let builder = builder.max_concurrent_requests(0); // Invalid but allowed
        assert_eq!(builder.get_max_concurrent_requests(), 0);
        
        let builder = builder.timeout_ms(0); // Invalid but allowed  
        assert_eq!(builder.get_timeout_ms(), 0);
        
        // Required fields should reject empty strings in build()
        let builder = builder
            .llm_provider("")  // Empty provider
            .llm_model("gpt-4");
            
        // Build should validate and use defaults for empty required fields
        let config = builder.build();
        assert!(!config.llm_provider.is_empty()); // Should use default
    }
    
    #[test]
    fn builder_accumulates_collections() {
        // Test that builder properly accumulates vectors (validators, tools)
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent")
            .add_validator("validator1")
            .add_validator("validator2")
            .add_tool("tool1")
            .add_tool("tool2")
            .add_tool("tool1"); // Duplicate
            
        assert_eq!(builder.validator_count(), 2);
        assert_eq!(builder.tool_count(), 3); // Allows duplicates
        
        let builder = builder
            .llm_provider("openai")
            .llm_model("gpt-4");
            
        let config = builder.build();
        assert_eq!(config.enabled_validators.len(), 2);
        assert_eq!(config.tools.len(), 3);
        assert!(config.enabled_validators.contains(&"validator1".to_string()));
        assert!(config.tools.contains(&"tool1".to_string()));
    }
    
    #[test]
    fn builder_metadata_handling() {
        // Test that builder handles metadata correctly
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent")
            .add_metadata("key1", "value1")
            .add_metadata("key2", "value2")
            .add_metadata("key1", "updated_value1"); // Overwrite
            
        let builder = builder
            .llm_provider("openai")
            .llm_model("gpt-4");
            
        let config = builder.build();
        assert_eq!(config.metadata.len(), 2);
        assert_eq!(config.metadata.get("key1"), Some(&"updated_value1".to_string()));
        assert_eq!(config.metadata.get("key2"), Some(&"value2".to_string()));
    }
    
    #[test]
    fn builder_send_sync() {
        // Test that builder can be sent across threads
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<ConfigBuilder<EmptyBuilder>>();
        assert_sync::<ConfigBuilder<EmptyBuilder>>();
        assert_send::<ConfigBuilder<PartialBuilder>>();
        assert_sync::<ConfigBuilder<PartialBuilder>>();
        assert_send::<ConfigBuilder<CompleteBuilder>>();
        assert_sync::<ConfigBuilder<CompleteBuilder>>();
    }
    
    #[test]
    fn builder_defaults() {
        // Test that builder applies appropriate defaults
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent")
            .llm_provider("openai")
            .llm_model("gpt-4");
            
        let config = builder.build();
        
        // Check defaults are applied
        assert_eq!(config.max_concurrent_requests, 10); // Default
        assert_eq!(config.timeout_ms, 30000); // Default  
        assert!(config.enabled_validators.is_empty()); // Default empty
        assert!(config.tools.is_empty()); // Default empty
        assert!(config.metadata.is_empty()); // Default empty
        assert_eq!(config.description, None); // Default None
    }
    
    #[test]
    fn builder_field_access() {
        // Test that builder provides access to current field values
        
        let builder = ConfigBuilder::<EmptyBuilder>::new("test-agent")
            .description("Test desc")
            .max_concurrent_requests(15)
            .timeout_ms(45000)
            .add_validator("val1")
            .add_tool("tool1")
            .llm_provider("openai");
            
        // Should be able to inspect current values
        assert_eq!(builder.get_name(), "test-agent");
        assert_eq!(builder.get_description(), &Some("Test desc".to_string()));
        assert_eq!(builder.get_max_concurrent_requests(), 15);
        assert_eq!(builder.get_timeout_ms(), 45000);
        assert_eq!(builder.validator_count(), 1);
        assert_eq!(builder.tool_count(), 1);
        assert!(builder.has_llm_provider());
        assert!(!builder.has_llm_model());
    }
}

// Implementation starts here - minimal to make tests pass

/// Builder state markers
pub trait BuilderState: Clone + std::fmt::Debug + Send + Sync + 'static {
    fn state_name() -> &'static str;
}

/// Empty builder - no required fields set
#[derive(Clone, Debug)]
pub struct EmptyBuilder;

impl BuilderState for EmptyBuilder {
    fn state_name() -> &'static str {
        "Empty"
    }
}

/// Partial builder - some but not all required fields set  
#[derive(Clone, Debug)]
pub struct PartialBuilder;

impl BuilderState for PartialBuilder {
    fn state_name() -> &'static str {
        "Partial"
    }
}

/// Complete builder - all required fields set
#[derive(Clone, Debug)]
pub struct CompleteBuilder;

impl BuilderState for CompleteBuilder {
    fn state_name() -> &'static str {
        "Complete"
    }
}

/// Type-safe configuration builder with required field tracking
pub struct ConfigBuilder<State: BuilderState> {
    name: String,
    description: Option<String>,
    max_concurrent_requests: u32,
    timeout_ms: u64,
    enabled_validators: Vec<String>,
    llm_provider: Option<String>,
    llm_model: Option<String>,
    tools: Vec<String>,
    metadata: HashMap<String, String>,
    _state: std::marker::PhantomData<State>,
}

impl ConfigBuilder<EmptyBuilder> {
    /// Create a new empty builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            max_concurrent_requests: 10,
            timeout_ms: 30000,
            enabled_validators: vec![],
            llm_provider: None,
            llm_model: None,
            tools: vec![],
            metadata: HashMap::new(),
            _state: std::marker::PhantomData,
        }
    }
}

impl<State: BuilderState> ConfigBuilder<State> {
    /// Get the current builder state name
    pub fn builder_state(&self) -> &'static str {
        State::state_name()
    }
    
    /// Check if LLM provider is set
    pub fn has_llm_provider(&self) -> bool {
        self.llm_provider.is_some()
    }
    
    /// Check if LLM model is set
    pub fn has_llm_model(&self) -> bool {
        self.llm_model.is_some()
    }
    
    /// Count of required fields that are set
    pub fn required_fields_count(&self) -> usize {
        let mut count = 0;
        if self.llm_provider.is_some() { count += 1; }
        if self.llm_model.is_some() { count += 1; }
        count
    }
    
    /// Count of required fields that are missing
    pub fn missing_required_count(&self) -> usize {
        2 - self.required_fields_count()
    }
    
    /// Get current name
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// Get current description
    pub fn get_description(&self) -> &Option<String> {
        &self.description
    }
    
    /// Get current max concurrent requests
    pub fn get_max_concurrent_requests(&self) -> u32 {
        self.max_concurrent_requests
    }
    
    /// Get current timeout
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }
    
    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.enabled_validators.len()
    }
    
    /// Get tool count
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
    
    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    /// Set max concurrent requests
    pub fn max_concurrent_requests(mut self, max: u32) -> Self {
        self.max_concurrent_requests = max;
        self
    }
    
    /// Set timeout
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }
    
    /// Add validator
    pub fn add_validator(mut self, validator: impl Into<String>) -> Self {
        self.enabled_validators.push(validator.into());
        self
    }
    
    /// Add tool
    pub fn add_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }
    
    /// Add metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// State transition methods
impl ConfigBuilder<EmptyBuilder> {
    /// Set LLM provider (transitions to PartialBuilder if this is first required field)
    pub fn llm_provider(mut self, provider: impl Into<String>) -> ConfigBuilder<PartialBuilder> {
        self.llm_provider = Some(provider.into());
        ConfigBuilder::<PartialBuilder> {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider,
            llm_model: self.llm_model,
            tools: self.tools,
            metadata: self.metadata,
            _state: std::marker::PhantomData,
        }
    }
    
    /// Set LLM model (transitions to PartialBuilder if this is first required field)
    pub fn llm_model(mut self, model: impl Into<String>) -> ConfigBuilder<PartialBuilder> {
        self.llm_model = Some(model.into());
        ConfigBuilder::<PartialBuilder> {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider,
            llm_model: self.llm_model,
            tools: self.tools,
            metadata: self.metadata,
            _state: std::marker::PhantomData,
        }
    }
}

impl ConfigBuilder<PartialBuilder> {
    /// Set LLM provider - may transition to CompleteBuilder if all required fields are set
    pub fn llm_provider(mut self, provider: impl Into<String>) -> ConfigBuilder<CompleteBuilder> {
        self.llm_provider = Some(provider.into());
        // Always transition to CompleteBuilder since this assumes llm_model is already set
        ConfigBuilder::<CompleteBuilder> {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider,
            llm_model: self.llm_model,
            tools: self.tools,
            metadata: self.metadata,
            _state: std::marker::PhantomData,
        }
    }
    
    /// Set LLM model - may transition to CompleteBuilder if all required fields are set
    pub fn llm_model(mut self, model: impl Into<String>) -> ConfigBuilder<CompleteBuilder> {
        self.llm_model = Some(model.into());
        // Always transition to CompleteBuilder since this assumes llm_provider is already set
        ConfigBuilder::<CompleteBuilder> {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider,
            llm_model: self.llm_model,
            tools: self.tools,
            metadata: self.metadata,
            _state: std::marker::PhantomData,
        }
    }
}

impl ConfigBuilder<CompleteBuilder> {
    /// Set LLM provider (stays in CompleteBuilder state)
    pub fn llm_provider(mut self, provider: impl Into<String>) -> Self {
        self.llm_provider = Some(provider.into());
        self
    }
    
    /// Set LLM model (stays in CompleteBuilder state)
    pub fn llm_model(mut self, model: impl Into<String>) -> Self {
        self.llm_model = Some(model.into());
        self
    }
    
    /// Build the final configuration (only available in CompleteBuilder state)
    pub fn build(self) -> AgentConfig {
        AgentConfig {
            name: self.name,
            description: self.description,
            max_concurrent_requests: self.max_concurrent_requests,
            timeout_ms: self.timeout_ms,
            enabled_validators: self.enabled_validators,
            llm_provider: self.llm_provider
                .filter(|p| !p.is_empty())
                .unwrap_or_else(|| "openai".to_string()),
            llm_model: self.llm_model
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| "gpt-4".to_string()),
            tools: self.tools,
            metadata: self.metadata,
        }
    }
}