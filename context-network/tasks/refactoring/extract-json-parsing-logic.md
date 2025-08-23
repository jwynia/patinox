# Extract Common JSON Parsing Logic

## Context
During code review of the service discovery implementation, duplicate JSON parsing patterns were identified in `parse_ollama_models()` and `parse_lmstudio_models()` methods.

## Current State
Both methods have similar error handling for JSON parsing:
```rust
let json: Value = serde_json::from_str(response)
    .map_err(|e| super::error::LocalProviderError::ParseError(
        format!("Failed to parse {} response: {}", service_name, e)
    ))?;
```

## Objective  
Reduce code duplication by extracting common JSON parsing logic while preserving the different field access patterns:
- Ollama: `json["models"][].name`
- LMStudio: `json["data"][].id`

## Acceptance Criteria
- [ ] Create helper method `parse_json_response(&self, response: &str, service_name: &str) -> LocalProviderResult<Value>`
- [ ] Both model parsing methods use the common helper
- [ ] All existing tests continue to pass
- [ ] Error messages maintain current quality and specificity

## Implementation Notes
- Consider making the helper generic over the service type
- Ensure error messages still identify which service failed
- Keep the field extraction logic separate (since schemas differ)

## Estimated Effort
30-45 minutes

## Priority
Medium - Code quality improvement, no functional impact