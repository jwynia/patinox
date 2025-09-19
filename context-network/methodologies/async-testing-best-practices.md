# Async Testing Best Practices

## Overview

Comprehensive guide for testing asynchronous operations in Rust, focusing on reliability, performance, and maintainability.

## Core Principles

### 1. Deterministic Testing
- Avoid time-based race conditions
- Use controlled async environments
- Mock external dependencies consistently

### 2. Isolation and Independence
- Each test should be independent
- No shared mutable state between tests
- Clean setup and teardown for each test

### 3. Realistic Async Scenarios
- Test actual async behavior, not just sync equivalents
- Include concurrency scenarios
- Test timeout and cancellation behavior

## Testing Patterns

### Basic Async Test Structure
```rust
#[tokio::test]
async fn test_async_operation() {
    // Setup
    let service = create_test_service().await;

    // Execute
    let result = service.async_operation().await;

    // Verify
    assert!(result.is_ok());
}
```

### Timeout Testing
```rust
#[tokio::test]
async fn test_operation_timeout() {
    let service = slow_service();

    let result = tokio::time::timeout(
        Duration::from_millis(100),
        service.slow_operation()
    ).await;

    assert!(result.is_err()); // Should timeout
}
```

### Concurrent Testing
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let service = Arc::new(create_service());

    let tasks: Vec<_> = (0..10)
        .map(|i| {
            let service = Arc::clone(&service);
            tokio::spawn(async move {
                service.concurrent_operation(i).await
            })
        })
        .collect();

    let results = join_all(tasks).await;

    // Verify all operations completed successfully
    for result in results {
        assert!(result.unwrap().is_ok());
    }
}
```

## Mock Strategies

### External Service Mocking
```rust
#[async_trait]
trait ExternalService {
    async fn call_api(&self, request: Request) -> Result<Response>;
}

struct MockExternalService {
    responses: VecDeque<Result<Response>>,
}

#[async_trait]
impl ExternalService for MockExternalService {
    async fn call_api(&self, _request: Request) -> Result<Response> {
        // Return predetermined responses
        self.responses.pop_front().unwrap_or(Err(Error::NetworkTimeout))
    }
}
```

### Time Mocking
```rust
use tokio_test::time::{self, Instant};

#[tokio::test]
async fn test_scheduled_operation() {
    time::pause();

    let mut service = Service::new();
    let task = tokio::spawn(async move {
        service.scheduled_operation().await
    });

    // Advance time without actually waiting
    time::advance(Duration::from_secs(60)).await;

    let result = task.await.unwrap();
    assert!(result.is_ok());
}
```

## Error Scenario Testing

### Network Failures
```rust
#[tokio::test]
async fn test_network_failure_recovery() {
    let mut mock_service = MockExternalService::new();
    mock_service.add_failure(NetworkError::ConnectionTimeout);
    mock_service.add_success(valid_response());

    let service = Service::with_external(mock_service);

    // Should retry and eventually succeed
    let result = service.resilient_operation().await;
    assert!(result.is_ok());
}
```

### Resource Exhaustion
```rust
#[tokio::test]
async fn test_resource_exhaustion() {
    let service = Service::with_limited_resources(1);

    // Start operation that consumes the resource
    let _ongoing = service.resource_intensive_operation();

    // Second operation should fail gracefully
    let result = service.resource_intensive_operation().await;
    assert_matches!(result, Err(Error::ResourceExhausted));
}
```

## Performance Testing

### Load Testing
```rust
#[tokio::test]
async fn test_service_under_load() {
    let service = Arc::new(Service::new());
    let start = Instant::now();

    let tasks: Vec<_> = (0..1000)
        .map(|_| {
            let service = Arc::clone(&service);
            tokio::spawn(async move {
                service.operation().await
            })
        })
        .collect();

    let results = join_all(tasks).await;
    let duration = start.elapsed();

    // Verify performance characteristics
    assert!(duration < Duration::from_secs(5));
    assert!(results.iter().all(|r| r.as_ref().unwrap().is_ok()));
}
```

### Memory Usage Testing
```rust
#[tokio::test]
async fn test_memory_usage() {
    let initial_memory = get_memory_usage();

    let service = Service::new();
    for _ in 0..1000 {
        service.operation().await.unwrap();
    }

    let final_memory = get_memory_usage();
    let memory_growth = final_memory - initial_memory;

    // Verify no significant memory leaks
    assert!(memory_growth < ACCEPTABLE_MEMORY_GROWTH);
}
```

## Test Organization

### Test Modules
```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod unit_tests {
        // Individual component tests
    }

    mod integration_tests {
        // Service interaction tests
    }

    mod performance_tests {
        // Load and benchmark tests
    }

    mod error_scenario_tests {
        // Failure mode tests
    }
}
```

### Test Helpers
```rust
mod test_helpers {
    pub async fn create_test_service() -> Service {
        Service::builder()
            .with_test_config()
            .with_mock_dependencies()
            .build()
            .await
    }

    pub fn assert_error_type<T>(result: Result<T>, expected_error: ErrorType) {
        match result {
            Err(error) if error.error_type() == expected_error => {},
            _ => panic!("Expected error type: {:?}", expected_error),
        }
    }
}
```

## Common Pitfalls

### Avoid These Anti-Patterns
- Using `thread::sleep()` in async tests
- Not properly mocking async dependencies
- Ignoring cancellation scenarios
- Testing only happy paths
- Sharing mutable state between tests

### Debug Techniques
- Use `tokio-console` for runtime debugging
- Add trace logging to async operations
- Use `tokio::time::pause()` for deterministic timing
- Implement custom debug formatters for complex async types

## Related Documentation

- [Error-Driven Development](error-driven-development.md)
- [Validation TDD Methodology](validation-tdd-methodology.md)
- [Concurrent Testing Strategies](concurrent-testing-strategies.md)

## Success Metrics

- **Test Reliability**: 99.9% test pass rate in CI
- **Test Speed**: Full async test suite under 2 minutes
- **Coverage**: 95%+ coverage of async code paths
- **Maintainability**: Clear, documented test patterns