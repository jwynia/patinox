# Fix Property-Based Test with Weak Properties

## Problem
The current property-based test in `tests/resource_management_test.rs:433-441` tests tautological properties that don't provide meaningful verification of the system behavior.

## Current Implementation
```rust
proptest! {
    #[test]
    fn test_resource_id_generation(id in any::<u32>()) {
        // Property: every resource should get a unique ID when created
        let resource1 = TestResource::new(id);
        let resource2 = TestResource::new(id);
        
        // Resource content might be same, but each should be cleanable
        prop_assert_eq!(resource1.id, resource2.id); // Same input, same output - tautological
        prop_assert!(!resource1.is_cleaned_up());
        prop_assert!(!resource2.is_cleaned_up());
    }
}
```

## Issues
- Tests that same input produces same output (tautological)
- Doesn't test any meaningful system properties
- Comment suggests testing ResourceId uniqueness but doesn't actually do it
- Properties tested are trivial constructor behavior

## Proposed Solutions

### Option 1: Test ResourceId Uniqueness Property
```rust
proptest! {
    #[test]
    fn resource_ids_should_be_unique_across_multiple_guards(resources in prop::collection::vec(any::<u32>(), 1..100)) {
        let mut resource_ids = std::collections::HashSet::new();
        
        for resource_data in resources {
            let resource = TestResource::new(resource_data);
            let guard = AsyncResourceGuard::new(resource, |r| async move { r.cleanup().await });
            
            let id = guard.resource_id();
            prop_assert!(resource_ids.insert(id), "ResourceId should be unique: {}", id);
        }
    }
}
```

### Option 2: Test Resource State Properties
```rust
proptest! {
    #[test]
    fn resources_should_maintain_state_invariants(id in any::<u32>(), should_fail in any::<bool>()) {
        let resource = if should_fail {
            TestResource::with_cleanup_failure(id)
        } else {
            TestResource::new(id)
        };
        
        // Property: Resources start in non-cleaned-up state
        prop_assert!(!resource.is_cleaned_up());
        prop_assert_eq!(resource.id, id);
        
        // Property: Failed resources remain in original state
        if should_fail {
            prop_assert_eq!(resource.should_fail_cleanup, true);
        }
    }
}
```

### Option 3: Remove Weak Property Tests
```rust
// Remove the tautological tests and keep only meaningful property tests
// or replace with focused unit tests that test specific behaviors
```

## Acceptance Criteria
- [ ] Remove or fix tautological property assertions
- [ ] Replace with meaningful properties that test system behavior
- [ ] Ensure properties can actually fail if implementation is wrong
- [ ] Add documentation explaining what properties are being tested
- [ ] Verify tests provide value over existing unit tests

## Analysis Required
- [ ] Determine what meaningful properties exist for ResourceId generation
- [ ] Identify what invariants should hold for TestResource behavior
- [ ] Evaluate whether property-based testing adds value over unit tests
- [ ] Consider if AsyncResourceGuard has testable properties worth verifying

## Implementation Notes
- Properties should test invariants that hold regardless of input values
- Tests should be able to fail if the implementation has bugs
- Consider whether property-based testing is the right approach here
- May be better to replace with focused unit tests if no good properties exist

## Priority: Medium
**Risk**: Low (test improvement only)
**Impact**: Better test coverage and more meaningful verification
**Effort**: 1-2 hours including analysis and implementation