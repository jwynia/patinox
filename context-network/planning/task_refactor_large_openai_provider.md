# Task: Refactor Large OpenAI Provider Structure

## Status
- **Priority**: Low
- **Complexity**: Medium
- **Effort**: Medium  
- **Dependencies**: None

## Context
During code review, the `src/provider/openai.rs` file was identified as becoming large (~500 lines) and containing multiple responsibilities that could be better separated.

## Problem Statement
The OpenAI provider implementation mixes several concerns:
1. **HTTP client management** and request formatting
2. **Model capability definitions** and hardcoded configurations
3. **API request/response transformation** between internal and OpenAI formats
4. **Provider-specific types** and serialization structures

This makes the code harder to maintain and test in isolation.

## Requirements
1. **Separation of Concerns**: Break into focused modules
2. **Testability**: Enable unit testing of individual components
3. **Maintainability**: Clearer structure for future modifications
4. **Reusability**: Common patterns that could benefit other providers

## Proposed Structure

### Module Organization
```
src/provider/openai/
├── mod.rs           # Public interface and main provider
├── client.rs        # HTTP client and request handling
├── types.rs         # OpenAI-specific request/response types
├── models.rs        # Model capabilities and configuration
└── transform.rs     # Format conversion utilities
```

### Responsibility Separation
1. **mod.rs**: Core `OpenAIProvider` struct and `ModelProvider` implementation
2. **client.rs**: HTTP client, header creation, error handling
3. **types.rs**: OpenAI API types (requests, responses, serialization)
4. **models.rs**: Model definitions and capability mapping
5. **transform.rs**: Convert between internal and OpenAI formats

## Implementation Plan

### Phase 1: Extract Types
- Move OpenAI-specific structs to `types.rs`
- Update imports and dependencies

### Phase 2: Extract Client Logic  
- Move HTTP client functionality to `client.rs`
- Abstract request/response handling

### Phase 3: Extract Model Configuration
- Move model capabilities to `models.rs`
- Prepare for externalization (future task)

### Phase 4: Extract Transformations
- Move format conversion logic to `transform.rs`
- Create reusable conversion utilities

## Benefits
1. **Improved Maintainability**: Smaller, focused files
2. **Better Testability**: Unit test individual components
3. **Code Reuse**: Common patterns for other providers
4. **Easier Debugging**: Clearer separation of concerns
5. **Future Extensibility**: Easier to add new features

## Acceptance Criteria
- [ ] OpenAI provider split into focused modules
- [ ] All existing functionality preserved
- [ ] No change to public API
- [ ] All tests continue to pass
- [ ] Each module has clear, single responsibility
- [ ] Internal documentation updated

## Files to Create
- `src/provider/openai/mod.rs`
- `src/provider/openai/client.rs`
- `src/provider/openai/types.rs`
- `src/provider/openai/models.rs`  
- `src/provider/openai/transform.rs`

## Files to Modify
- `src/provider/mod.rs` - Update imports
- `src/provider/openai.rs` - Convert to directory module

## Testing Strategy
- Ensure all existing tests continue to pass
- Add unit tests for individual modules
- Test module boundaries and interfaces
- Verify no performance regression

## Notes
This refactoring improves code organization without changing functionality. It should be done carefully with comprehensive testing to ensure no behavioral changes. The modular structure will also make it easier to implement similar providers (Anthropic, etc.) in the future.

Created: 2025-01-20 (deferred from code review)