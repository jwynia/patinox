# Comprehensive Testing Strategy for Patinox

## Overview

This document defines a comprehensive testing strategy for the Patinox agent framework, addressing the unique challenges of testing AI agents, non-deterministic behaviors, and external service dependencies.

## Testing Challenges

### Unique to AI Agent Systems
1. **Non-deterministic outputs**: LLMs produce different outputs for same inputs
2. **External API dependencies**: Expensive and rate-limited LLM APIs
3. **Long-running workflows**: Agent tasks can take minutes or hours
4. **Complex state machines**: Multiple execution paths and decision points
5. **Emergent behaviors**: Unexpected interactions between components
6. **Cost considerations**: Testing with real APIs is expensive

## Testing Pyramid

```
        /\
       /  \  E2E Tests (5%)
      /    \  - Full system tests
     /      \  - Production-like environment
    /--------\
   /          \ Integration Tests (15%)
  /            \ - Component interaction
 /              \ - API contract tests
/________________\
     Unit Tests (80%)
   - Fast, isolated, deterministic
   - Core logic validation
```

## Testing Layers

### 1. Unit Testing

#### Core Testing Principles
```rust
// Every public function should have tests
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_agent_creation() {
        // Arrange
        let deps = MockDependencies::new();
        
        // Act
        let agent = Agent::with_dependencies(deps);
        
        // Assert
        assert_eq!(agent.id().to_string().len(), 36); // UUID
        assert_eq!(agent.state(), AgentState::Ready);
    }
    
    // Property-based testing for complex logic
    proptest! {
        #[test]
        fn test_prompt_validation_never_panics(
            prompt in ".*",
            max_length in 1..10000usize
        ) {
            let validator = PromptValidator::new(max_length);
            // Should never panic, only return Result
            let _ = validator.validate(&prompt);
        }
    }
}
```

#### Mocking Strategy
```rust
/// Mock LLM for testing
#[automock]
pub trait LanguageModel: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_agent_with_mock_llm() {
        let mut mock_llm = MockLanguageModel::new();
        
        // Set up expectations
        mock_llm
            .expect_complete()
            .with(predicate::str::contains("test"))
            .times(1)
            .returning(|_| Ok("Mocked response".to_string()));
        
        let agent = Agent::builder()
            .with_llm(Box::new(mock_llm))
            .build();
            
        let result = agent.execute("test task").await;
        assert!(result.is_ok());
    }
}
```

### 2. Integration Testing

#### Component Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use testcontainers::{clients, images};
    
    #[tokio::test]
    async fn test_agent_with_vector_db() {
        // Start Qdrant in container
        let docker = clients::Cli::default();
        let qdrant = docker.run(images::qdrant::Qdrant::default());
        
        let qdrant_url = format!("http://localhost:{}", qdrant.get_host_port(6333));
        
        // Create agent with real vector DB
        let agent = Agent::builder()
            .with_memory(QdrantMemory::new(&qdrant_url))
            .build();
        
        // Test memory operations
        agent.remember("test fact").await.unwrap();
        let recalled = agent.recall("test").await.unwrap();
        assert!(recalled.contains("test fact"));
    }
}
```

#### Contract Testing
```rust
/// Ensure our API matches the spec
#[cfg(test)]
mod contract_tests {
    use pact_consumer::*;
    
    #[tokio::test]
    async fn test_llm_provider_contract() {
        let pact = PactBuilder::new("patinox", "openai")
            .interaction("completion request", |i| {
                i.given("API key is valid")
                 .request("POST", "/v1/completions")
                 .json_body(json!({
                     "model": "gpt-4",
                     "prompt": "Hello",
                     "max_tokens": 100
                 }))
                 .response(200)
                 .json_body(json!({
                     "choices": [{
                         "text": "Hello! How can I help you?",
                         "index": 0
                     }]
                 }))
            })
            .build();
            
        // Test our client against the contract
        let client = OpenAIClient::new("test-key");
        let result = client.complete("Hello").await;
        assert!(result.is_ok());
        
        pact.verify().await;
    }
}
```

### 3. End-to-End Testing

#### Scenario-Based Testing
```rust
#[cfg(test)]
mod e2e_tests {
    use cucumber::{given, when, then, World};
    
    #[derive(Debug, Default, World)]
    struct AgentWorld {
        agent: Option<Agent>,
        result: Option<Result<Value>>,
    }
    
    #[given("an agent with research capabilities")]
    async fn create_research_agent(world: &mut AgentWorld) {
        world.agent = Some(
            Agent::builder()
                .with_paradigm(Paradigm::TreeOfThoughts)
                .with_tools(vec![WebSearch::new(), Scholar::new()])
                .build()
        );
    }
    
    #[when("I ask it to research {string}")]
    async fn research_topic(world: &mut AgentWorld, topic: String) {
        let agent = world.agent.as_ref().unwrap();
        world.result = Some(agent.research(&topic).await);
    }
    
    #[then("it should return relevant findings")]
    async fn check_findings(world: &mut AgentWorld) {
        let result = world.result.as_ref().unwrap();
        assert!(result.is_ok());
        let findings = result.as_ref().unwrap();
        assert!(findings.get("sources").unwrap().as_array().unwrap().len() > 0);
    }
}
```

### 4. Performance Testing

#### Benchmark Suite
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_creation(c: &mut Criterion) {
    c.bench_function("agent creation", |b| {
        b.iter(|| {
            Agent::builder()
                .with_defaults()
                .build()
        })
    });
}

fn bench_validation_pipeline(c: &mut Criterion) {
    let validator = create_validation_pipeline();
    let input = "Test prompt that needs validation";
    
    c.bench_function("validation pipeline", |b| {
        b.iter(|| {
            validator.validate(black_box(input))
        })
    });
}

criterion_group!(benches, bench_agent_creation, bench_validation_pipeline);
criterion_main!(benches);
```

#### Load Testing
```rust
/// Load test using drill or custom harness
#[cfg(test)]
mod load_tests {
    use tokio::task::JoinSet;
    
    #[tokio::test]
    async fn test_concurrent_agent_execution() {
        let agent = Arc::new(create_test_agent());
        let mut tasks = JoinSet::new();
        
        // Simulate 100 concurrent requests
        for i in 0..100 {
            let agent = agent.clone();
            tasks.spawn(async move {
                let start = Instant::now();
                let result = agent.execute(&format!("Task {}", i)).await;
                let duration = start.elapsed();
                (result, duration)
            });
        }
        
        let mut successes = 0;
        let mut total_duration = Duration::ZERO;
        
        while let Some(result) = tasks.join_next().await {
            let (result, duration) = result.unwrap();
            if result.is_ok() {
                successes += 1;
            }
            total_duration += duration;
        }
        
        assert!(successes >= 95); // 95% success rate
        assert!(total_duration.as_secs() < 60); // All complete within 1 minute
    }
}
```

### 5. Specialized Testing

#### Deterministic Testing for Non-Deterministic Systems
```rust
/// Test non-deterministic behaviors deterministically
pub struct DeterministicTestHarness {
    seed: u64,
    recorded_responses: HashMap<String, String>,
}

impl DeterministicTestHarness {
    /// Record mode: save real responses
    pub async fn record_test_case(&mut self, name: &str) {
        let agent = create_real_agent();
        let result = agent.execute("test prompt").await.unwrap();
        self.recorded_responses.insert(name.to_string(), result);
        self.save_recordings();
    }
    
    /// Replay mode: use recorded responses
    pub async fn replay_test_case(&self, name: &str) {
        let response = self.recorded_responses.get(name).unwrap();
        let mock_llm = MockLLM::with_response(response);
        
        let agent = Agent::builder()
            .with_llm(mock_llm)
            .build();
            
        let result = agent.execute("test prompt").await.unwrap();
        assert_eq!(result, response);
    }
}
```

#### Fuzzing
```rust
use afl::fuzz;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(input) = std::str::from_utf8(data) {
            let validator = PromptValidator::new();
            // Should never panic, only return errors
            let _ = validator.validate(input);
        }
    });
}
```

#### Chaos Testing
```rust
/// Chaos testing for resilience
#[cfg(test)]
mod chaos_tests {
    use chaos_monkey::*;
    
    #[tokio::test]
    async fn test_agent_resilience() {
        let chaos = ChaosMonkey::new()
            .with_network_delays(0.1)  // 10% chance of network delay
            .with_service_failures(0.05)  // 5% chance of service failure
            .with_memory_pressure(0.02);  // 2% chance of memory pressure
        
        chaos.unleash();
        
        let agent = create_test_agent();
        let mut successes = 0;
        
        for _ in 0..100 {
            if agent.execute("test").await.is_ok() {
                successes += 1;
            }
        }
        
        // Should maintain 80% success rate even under chaos
        assert!(successes >= 80);
    }
}
```

## Test Data Management

### Fixtures
```rust
/// Reusable test fixtures
pub mod fixtures {
    use once_cell::sync::Lazy;
    
    pub static TEST_AGENT: Lazy<Agent> = Lazy::new(|| {
        Agent::builder()
            .with_test_config()
            .build()
    });
    
    pub fn sample_prompts() -> Vec<String> {
        vec![
            "Simple question".to_string(),
            "Complex multi-part question with context".to_string(),
            "Edge case with special characters: 🎉 \n \t".to_string(),
        ]
    }
    
    pub fn expected_responses() -> HashMap<String, String> {
        // Load from test data files
        load_test_data("tests/data/responses.json")
    }
}
```

### Test Doubles
```rust
/// Different test double strategies
pub mod test_doubles {
    /// Dummy - returns default values
    pub struct DummyLLM;
    
    impl LanguageModel for DummyLLM {
        async fn complete(&self, _: &str) -> Result<String> {
            Ok(String::new())
        }
    }
    
    /// Stub - returns predetermined values
    pub struct StubLLM {
        responses: Vec<String>,
        index: AtomicUsize,
    }
    
    impl LanguageModel for StubLLM {
        async fn complete(&self, _: &str) -> Result<String> {
            let i = self.index.fetch_add(1, Ordering::SeqCst);
            Ok(self.responses[i % self.responses.len()].clone())
        }
    }
    
    /// Spy - records interactions
    pub struct SpyLLM {
        pub calls: Arc<Mutex<Vec<String>>>,
        delegate: Box<dyn LanguageModel>,
    }
    
    impl LanguageModel for SpyLLM {
        async fn complete(&self, prompt: &str) -> Result<String> {
            self.calls.lock().unwrap().push(prompt.to_string());
            self.delegate.complete(prompt).await
        }
    }
    
    /// Fake - simplified implementation
    pub struct FakeLLM {
        behavior: HashMap<String, String>,
    }
    
    impl LanguageModel for FakeLLM {
        async fn complete(&self, prompt: &str) -> Result<String> {
            // Simple pattern matching instead of real LLM
            for (pattern, response) in &self.behavior {
                if prompt.contains(pattern) {
                    return Ok(response.clone());
                }
            }
            Ok("Default response".to_string())
        }
    }
}
```

## CI/CD Integration

### Test Pipeline
```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run unit tests
        run: cargo test --lib
      - name: Upload coverage
        run: cargo tarpaulin --out Xml
        
  integration-tests:
    runs-on: ubuntu-latest
    services:
      qdrant:
        image: qdrant/qdrant
        ports:
          - 6333:6333
    steps:
      - uses: actions/checkout@v2
      - name: Run integration tests
        run: cargo test --test '*' -- --test-threads=1
        
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup test environment
        run: docker-compose -f test-env.yml up -d
      - name: Run E2E tests
        run: cargo test --features e2e
        
  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run benchmarks
        run: cargo bench
      - name: Check for performance regressions
        run: cargo bench -- --save-baseline new && cargo bench -- --baseline main
```

## Test Coverage Goals

### Coverage Targets
- **Overall**: >= 80%
- **Core logic**: >= 90%
- **Error handling**: >= 95%
- **Security code**: 100%

### Coverage Enforcement
```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "instrument-coverage"]

[test]
rustflags = ["-C", "instrument-coverage"]
```

## Testing Best Practices

### 1. Test Naming
```rust
// Good test names describe behavior
#[test]
fn agent_should_retry_on_transient_network_failure() { }

// Not just the function name
#[test]
fn test_execute() { } // Bad
```

### 2. Test Independence
- Each test should be independent
- Use fresh fixtures for each test
- Clean up resources after tests

### 3. Test Speed
- Unit tests: < 10ms each
- Integration tests: < 1s each
- E2E tests: < 10s each

### 4. Test Documentation
```rust
/// Tests that the agent correctly handles rate limiting from the LLM provider.
/// 
/// This test simulates a rate limit error and verifies that the agent:
/// 1. Recognizes the rate limit
/// 2. Backs off appropriately
/// 3. Retries successfully
#[test]
fn handles_rate_limiting() {
    // Test implementation
}
```

## Continuous Testing

### Mutation Testing
```bash
# Use cargo-mutants to verify test quality
cargo mutants --all
```

### Test Impact Analysis
- Only run tests affected by changes
- Use dependency graph to determine impact
- Prioritize critical path tests

## Open Questions

1. How do we test agent-to-agent interactions?
2. What's the strategy for testing emergent behaviors?
3. How do we validate AI safety measures?
4. Should we use formal verification for critical paths?
5. What about testing in production (shadow mode)?

## References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Property Testing with PropTest](https://proptest-rs.github.io/proptest/)
- [Testing LLM Applications](https://www.anthropic.com/research/testing-llms)
- [Chaos Engineering Principles](https://principlesofchaos.org/)