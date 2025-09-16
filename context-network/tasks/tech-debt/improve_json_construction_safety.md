# Task: Improve JSON Construction Safety in Testing Utilities

## Status
- **Priority**: Medium
- **Complexity**: Small-Medium
- **Effort**: Small (30-45 minutes)
- **Risk**: Low

## Context

**Original Code Review Recommendation**: "JSON Construction Without Escaping - Manual JSON construction without proper escaping could lead to malformed JSON if input contains special characters."

**Current Issue**: The `MockHttpBuilder` methods use manual string formatting to create JSON responses, which could break if model names or response content contain special characters like quotes, backslashes, or newlines.

**Affected Files**:
- `tests/utils/mod.rs:145` - `with_models_response` method
- `tests/utils/mod.rs:156-157` - `with_completion_response` method

## Problem Statement

Current implementation uses unsafe string formatting:
```rust
.map(|model| format!(r#"{{"id":"{}","object":"model","created":{}}}"#, model, MOCK_TIMESTAMP))
```

This could produce malformed JSON if `model` contains characters like `"`, `\`, or control characters.

## Requirements

1. **Safe JSON Construction**: Replace manual string formatting with proper JSON serialization
2. **Maintain API**: Keep the same public interface for testing utilities
3. **Error Handling**: Handle JSON serialization errors appropriately
4. **Performance**: Maintain reasonable performance for test utilities

## Implementation Approach

1. Add `serde_json` to dev-dependencies (if not already present)
2. Replace manual formatting with `serde_json::json!` macro
3. Update both `with_models_response` and `with_completion_response` methods
4. Add tests to verify handling of special characters in input

## Acceptance Criteria

- [ ] Use `serde_json` for all JSON construction in test utilities
- [ ] Handle special characters correctly (quotes, backslashes, newlines)
- [ ] Maintain existing test compatibility
- [ ] Add test case with special characters in model names/content
- [ ] No performance regression in test execution

## Implementation Example

```rust
use serde_json::json;

pub fn with_models_response(self, models: &[&str]) -> Self {
    let models_json: Vec<_> = models.iter()
        .map(|model| json!({
            "id": model,
            "object": "model",
            "created": MOCK_TIMESTAMP
        }))
        .collect();
    let response_body = json!({"data": models_json}).to_string();

    MockHttpBuilder {
        endpoint: self.endpoint,
        status_code: self.status_code,
        error_message: Some(response_body),
        retry_after: self.retry_after,
    }
}
```

## Testing Plan

Add test case to verify special character handling:
```rust
#[test]
fn test_json_safety_with_special_characters() {
    let mock = MockHttpBuilder::new()
        .with_models_response(&["model\"with\"quotes", "model\nwith\nnewlines"])
        .build();

    // Should produce valid JSON
    let json: serde_json::Value = serde_json::from_str(mock.response_body()).expect("Valid JSON");
    // Verify content is properly escaped
}
```

## Notes

This is a good practice improvement that prevents potential issues in tests when dealing with model names that might contain special characters. While unlikely in current usage, it makes the testing utilities more robust and follows JSON construction best practices.

**Dependencies**: None
**Related Tasks**: None

Created: 2025-09-15 (Code review follow-up)