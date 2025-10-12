# Task: Externalize Model Configurations

## Status
- **Priority**: Medium  
- **Complexity**: Medium
- **Effort**: Small-Medium
- **Dependencies**: None

## Context
During code review of the LLM Provider Abstraction implementation, hardcoded model configurations were identified in the OpenAI provider implementation.

## Problem Statement
The `get_model_capabilities_by_name` method in `src/provider/openai.rs` contains hardcoded model specifications including:
- Model names and variants
- Token limits
- Cost per 1k tokens
- Capability flags (tools, vision, streaming support)

This creates maintenance overhead and reduces flexibility.

## Requirements
1. **External Configuration**: Move model configurations to external files (JSON/TOML)
2. **Runtime Loading**: Load configurations at provider initialization 
3. **Validation**: Validate configuration completeness and format
4. **Fallback**: Maintain reasonable defaults for unknown models
5. **Updates**: Allow configuration updates without code recompilation

## Implementation Approach
1. Create `provider/configs/` directory structure
2. Add `openai-models.toml` configuration file
3. Create `ModelConfigLoader` for loading and parsing configurations
4. Update `OpenAIProvider` to use loaded configurations
5. Add configuration validation with helpful error messages

## Acceptance Criteria
- [ ] Model configurations moved to external files
- [ ] Provider loads configurations at initialization
- [ ] Invalid configurations produce clear error messages
- [ ] Fallback behavior for unknown models
- [ ] Tests cover configuration loading scenarios
- [ ] Documentation updated with configuration format

## Files to Modify
- `src/provider/openai.rs` - Remove hardcoded configurations
- `src/provider/config.rs` - Add model configuration loading
- Create `provider/configs/openai-models.toml`
- Update tests and documentation

## Notes
This change improves maintainability and allows users to customize model configurations without code changes. Consider making this part of the broader configuration system.

Created: 2025-01-20 (deferred from code review)