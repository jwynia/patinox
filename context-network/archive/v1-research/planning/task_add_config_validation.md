# Task: Add Configuration Validation and Bounds Checking

**Created**: 2025-08-19 15:30 CDT
**Status**: Planned (deferred from code review recommendations)
**Priority**: Medium
**Category**: Validation / Robustness

## Overview

Add bounds checking and validation for configuration values across all framework components to prevent invalid configurations and improve system robustness.

## Context from Code Review

**Original Recommendation**: "Missing Validation: In `validator.rs:305`, consider adding bounds checking for config priority values."

**Why Deferred**: This requires systematic analysis of all configuration types and design of validation patterns. It's better to handle comprehensively rather than piecemeal.

## Scope

### Configuration Types to Validate:

#### 1. AgentConfig (`src/traits/agent.rs`)
- `max_concurrent_requests`: Should have reasonable bounds (1-1000)
- `timeout_ms`: Should be positive and reasonable (1000-300000ms) 
- `enabled_validators`: Should reference valid validators
- `tools`: Should reference available tools
- `llm_provider`/`llm_model`: Should be non-empty strings

#### 2. ValidatorConfig (`src/traits/validator.rs`)
- `priority`: Should have reasonable bounds (-100 to 100)
- `stages`: Should not be empty for enabled validators
- `parameters`: Should validate based on validator type

#### 3. MonitorConfig (`src/traits/monitor.rs`)  
- `buffer_size`: Should be positive (1-100000)
- `flush_interval_ms`: Should be positive (100-3600000ms)
- `sampling_rate`: Should be between 0.0 and 1.0
- `event_types`: Should not be empty if enabled

#### 4. ToolMetadata (`src/traits/tool.rs`)
- `version`: Should follow semantic versioning pattern
- `category`: Should be from allowed categories or non-empty
- `tags`: Should have reasonable limits on count and length

## Acceptance Criteria

- [ ] All configuration types have validation methods
- [ ] Validation provides clear, actionable error messages
- [ ] Bounds checking prevents obviously invalid configurations
- [ ] Validation is consistently applied during construction
- [ ] Builder patterns include validation steps
- [ ] Configuration deserialization includes validation
- [ ] Performance impact of validation is minimal

## Implementation Approach

### Phase 1: Validation Framework Design
- [ ] Define validation trait/pattern for configurations
- [ ] Establish error types for validation failures
- [ ] Design builder pattern integration
- [ ] Plan validation timing (construction vs usage)

### Phase 2: Core Configuration Validation
- [ ] Implement AgentConfig validation
- [ ] Implement ValidatorConfig validation  
- [ ] Implement MonitorConfig validation
- [ ] Implement ToolMetadata validation

### Phase 3: Integration and Testing
- [ ] Add validation to builder patterns
- [ ] Update tests to cover validation scenarios
- [ ] Test error messages for clarity
- [ ] Performance test validation overhead

### Phase 4: Documentation and Examples
- [ ] Document validation rules and rationale
- [ ] Add examples of valid and invalid configurations
- [ ] Update builder pattern documentation
- [ ] Add troubleshooting guide for validation errors

## Validation Patterns to Implement

### 1. Builder Pattern Validation
```rust
impl AgentBuilder {
    pub fn max_concurrent_requests(mut self, max: u32) -> Result<Self, ConfigurationError> {
        if max == 0 || max > 1000 {
            return Err(ConfigurationError::InvalidValue(
                "max_concurrent_requests must be between 1 and 1000".to_string()
            ));
        }
        self.config.max_concurrent_requests = max;
        Ok(self)
    }
}
```

### 2. Configuration Validation Trait
```rust
trait ValidateConfig {
    type Error;
    
    fn validate(&self) -> Result<(), Self::Error>;
    fn validate_field(field_name: &str, value: &dyn Any) -> Result<(), Self::Error>;
}
```

### 3. Serde Integration
```rust
impl<'de> Deserialize<'de> for AgentConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = AgentConfigRaw::deserialize(deserializer)?;
        config.validate().map_err(serde::de::Error::custom)?;
        Ok(config.into())
    }
}
```

## Validation Rules to Implement

### AgentConfig Validation:
```rust
fn validate_agent_config(config: &AgentConfig) -> Result<(), ConfigurationError> {
    // Concurrent requests bounds
    if config.max_concurrent_requests == 0 || config.max_concurrent_requests > 1000 {
        return Err(ConfigurationError::InvalidValue(
            "max_concurrent_requests must be between 1 and 1000".to_string()
        ));
    }
    
    // Timeout bounds (1 second to 5 minutes)
    if config.timeout_ms < 1000 || config.timeout_ms > 300000 {
        return Err(ConfigurationError::InvalidValue(
            "timeout_ms must be between 1000 and 300000".to_string()
        ));
    }
    
    // Required fields
    if config.name.trim().is_empty() {
        return Err(ConfigurationError::MissingRequired("name".to_string()));
    }
    
    if config.llm_provider.trim().is_empty() {
        return Err(ConfigurationError::MissingRequired("llm_provider".to_string()));
    }
    
    Ok(())
}
```

### ValidatorConfig Validation:
```rust
fn validate_validator_config(config: &ValidatorConfig) -> Result<(), ConfigurationError> {
    // Priority bounds
    if config.priority < -100 || config.priority > 100 {
        return Err(ConfigurationError::InvalidValue(
            "priority must be between -100 and 100".to_string()
        ));
    }
    
    // Must have stages if enabled
    if config.enabled && config.stages.is_empty() {
        return Err(ConfigurationError::InvalidValue(
            "enabled validator must specify at least one validation stage".to_string()
        ));
    }
    
    Ok(())
}
```

## Error Handling Strategy

### New Error Types:
```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigurationValidationError {
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
    
    #[error("Value {value} for {field} is out of bounds ({min} to {max})")]
    OutOfBounds { field: String, value: String, min: String, max: String },
    
    #[error("Required field {field} is missing or empty")]
    MissingRequired { field: String },
    
    #[error("Invalid format for {field}: {message}")]
    InvalidFormat { field: String, message: String },
}
```

## Testing Strategy

### Validation Test Patterns:
```rust
#[test]
fn test_agent_config_validation() {
    // Valid configuration should pass
    let valid_config = AgentConfig { /* valid values */ };
    assert!(valid_config.validate().is_ok());
    
    // Invalid concurrent requests should fail
    let invalid_config = AgentConfig {
        max_concurrent_requests: 0,
        ..valid_config.clone()
    };
    assert!(invalid_config.validate().is_err());
    
    // Error message should be helpful
    match invalid_config.validate() {
        Err(ConfigurationValidationError::OutOfBounds { field, .. }) => {
            assert_eq!(field, "max_concurrent_requests");
        }
        _ => panic!("Expected OutOfBounds error"),
    }
}
```

## Estimated Effort

**Size**: Medium (systematic validation across multiple configuration types)
**Timeline**: 2-3 hours  
**Risk**: Low (validation only, improves robustness)

## Dependencies

- Requires stable configuration types (current PR provides this)
- Should coordinate with builder pattern improvements
- May need new error types in error module
- Consider impact on existing tests

## Success Metrics

### Robustness Improvements:
- Elimination of obviously invalid configurations
- Clear error messages for configuration problems
- Reduced runtime errors from misconfiguration
- Improved developer experience with helpful validation

### Code Quality:
- Consistent validation patterns across all config types
- Comprehensive test coverage for validation scenarios
- Documentation of all validation rules and rationale
- Performance impact is negligible

## Related Context

- Builds on comprehensive error system from current PR
- Supports production readiness and operational reliability
- Improves developer experience and reduces configuration errors
- Enables better testing and validation of framework components