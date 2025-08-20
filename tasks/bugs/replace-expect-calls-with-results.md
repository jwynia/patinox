# Replace expect() calls with Result types in AsyncResourceGuard

## Problem
The AsyncResourceGuard methods `get()`, `get_mut()`, and `into_inner()` currently use `expect()` calls that can panic if the resource has already been consumed. This violates Rust best practices and can cause unexpected crashes in production.

## Current Implementation
```rust
pub fn get(&self) -> &T {
    self.resource.as_ref().expect("Resource was already consumed")
}

pub fn get_mut(&mut self) -> &mut T {
    self.resource.as_mut().expect("Resource was already consumed")
}

pub fn into_inner(mut self) -> T {
    self.resource.take().expect("Resource was already consumed")
}
```

## Proposed Solution Options

### Option 1: Return Result Types
```rust
pub fn get(&self) -> Result<&T, CleanupError> {
    self.resource.as_ref().ok_or(CleanupError::AlreadyCleanedUp)
}

pub fn get_mut(&mut self) -> Result<&mut T, CleanupError> {
    self.resource.as_mut().ok_or(CleanupError::AlreadyCleanedUp)
}

pub fn into_inner(mut self) -> Result<T, CleanupError> {
    self.resource.take().ok_or(CleanupError::AlreadyCleanedUp)
}
```

### Option 2: Return Option Types
```rust
pub fn get(&self) -> Option<&T> {
    self.resource.as_ref()
}

pub fn get_mut(&mut self) -> Option<&mut T> {
    self.resource.as_mut()
}

pub fn into_inner(mut self) -> Option<T> {
    self.resource.take()
}
```

## Acceptance Criteria
- [ ] Replace all expect() calls with safe alternatives
- [ ] Maintain existing functionality for valid use cases
- [ ] Update all existing tests to handle new API
- [ ] Add tests for error conditions
- [ ] Update documentation with usage examples
- [ ] Consider backward compatibility implications
- [ ] Ensure consistent error handling pattern across the codebase

## Implementation Notes
- This is a breaking change that affects the public API
- Need to audit all usage sites in tests and examples
- Consider providing both safe and unsafe variants for migration
- May need to update other components that depend on these methods

## Priority: High
**Risk**: API breaking change
**Impact**: Prevents production panics, improves API safety
**Effort**: 2-4 hours including testing and documentation