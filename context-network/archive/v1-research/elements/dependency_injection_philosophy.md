# Dependency Injection Philosophy: Freedom Without Obligation

## Overview

This document explores dependency injection (DI) patterns in Rust for the Patinox framework, focusing on the principle of "freedom to without the obligation to" - providing flexibility for customization while maintaining sensible defaults that work without configuration.

## Core Philosophy

### The Freedom-Without-Obligation Principle

Users should have the **freedom to** override any behavior at any level of the hierarchy, but never the **obligation to** provide implementations for standard use cases. This principle manifests across multiple dimensions:

1. **Design-time Freedom**: Choose implementations through traits
2. **Compile-time Freedom**: Configure through generics and const parameters
3. **Runtime Freedom**: Override through dynamic dispatch when needed
4. **Deployment Freedom**: Inject through configuration without recompilation

## Rust's Unique Approach to DI

Unlike traditional OOP languages, Rust's ownership system and trait model enable unique DI patterns:

### 1. Trait-Based Abstraction

```rust
// Core abstraction - what an agent needs
pub trait AgentDependencies: Send + Sync {
    type Model: LanguageModel;
    type Validator: Validator;
    type Monitor: Monitor;
    type Storage: Storage;
    
    fn model(&self) -> &Self::Model;
    fn validator(&self) -> &Self::Validator;
    fn monitor(&self) -> &Self::Monitor;
    fn storage(&self) -> &Self::Storage;
}

// Default implementation - works out of the box
pub struct DefaultDependencies {
    model: OpenRouterModel,
    validator: StandardValidator,
    monitor: TelemetryMonitor,
    storage: InMemoryStorage,
}

impl Default for DefaultDependencies {
    fn default() -> Self {
        // Sensible defaults that just work
        Self {
            model: OpenRouterModel::new_from_env(),
            validator: StandardValidator::safe_defaults(),
            monitor: TelemetryMonitor::new_otlp(),
            storage: InMemoryStorage::with_capacity(1000),
        }
    }
}
```

### 2. Builder Pattern with Optional Overrides

```rust
pub struct AgentBuilder<D = DefaultDependencies> {
    dependencies: D,
    // Optional overrides at agent level
    model_override: Option<Box<dyn LanguageModel>>,
    validator_override: Option<Box<dyn Validator>>,
}

impl AgentBuilder {
    // Start with defaults - no configuration required
    pub fn new() -> Self {
        Self {
            dependencies: DefaultDependencies::default(),
            model_override: None,
            validator_override: None,
        }
    }
    
    // Freedom to override specific components
    pub fn with_model<M: LanguageModel + 'static>(mut self, model: M) -> Self {
        self.model_override = Some(Box::new(model));
        self
    }
    
    // Freedom to use custom dependency container
    pub fn with_dependencies<D: AgentDependencies>(self, deps: D) -> AgentBuilder<D> {
        AgentBuilder {
            dependencies: deps,
            model_override: self.model_override,
            validator_override: self.validator_override,
        }
    }
}
```

### 3. Hierarchical Override Pattern

```rust
/// Hierarchical configuration with cascading overrides
pub struct HierarchicalDependencies {
    // Global level - from config files or environment
    global: Arc<dyn AgentDependencies>,
    
    // Agent level - specific to this agent instance
    agent: Option<Arc<dyn AgentDependencies>>,
    
    // Request level - per-request overrides
    request: Option<Arc<dyn AgentDependencies>>,
}

impl HierarchicalDependencies {
    /// Resolve dependencies with proper precedence
    pub fn resolve<T>(&self, 
        getter: impl Fn(&dyn AgentDependencies) -> Option<T>
    ) -> Option<T> {
        // Request overrides agent overrides global
        self.request.as_ref().and_then(|d| getter(d.as_ref()))
            .or_else(|| self.agent.as_ref().and_then(|d| getter(d.as_ref())))
            .or_else(|| getter(self.global.as_ref()))
    }
}
```

## Compile-Time vs Runtime Injection

### Compile-Time Injection (Zero-Cost)

```rust
// Generic over dependencies - resolved at compile time
pub struct Agent<D: AgentDependencies> {
    deps: D,
    state: AgentState,
}

impl<D: AgentDependencies> Agent<D> {
    pub async fn execute(&mut self, task: Task) -> Result<Output> {
        // Direct access to dependencies - no vtable lookup
        let model = self.deps.model();
        let validator = self.deps.validator();
        
        // Compiler can inline and optimize
        validator.validate(&task)?;
        let response = model.complete(&task).await?;
        Ok(response)
    }
}

// Usage - type is known at compile time
let agent: Agent<DefaultDependencies> = Agent::new();
let agent_custom: Agent<CustomDeps> = Agent::with_deps(custom_deps);
```

### Runtime Injection (Flexible)

```rust
// Dynamic dispatch when flexibility is needed
pub struct DynamicAgent {
    deps: Box<dyn AgentDependencies>,
    state: AgentState,
}

impl DynamicAgent {
    pub fn new() -> Self {
        Self::with_deps(Box::new(DefaultDependencies::default()))
    }
    
    pub fn with_deps(deps: Box<dyn AgentDependencies>) -> Self {
        Self { deps, state: AgentState::new() }
    }
    
    // Can swap dependencies at runtime
    pub fn replace_model(&mut self, model: Box<dyn LanguageModel>) {
        // Runtime flexibility when needed
        self.deps.set_model(model);
    }
}
```

## Service Locator Pattern

For complex dependency graphs:

```rust
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    pub fn register<T: Any + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }
    
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref())
    }
    
    // Builder-style registration
    pub fn with<T: Any + Send + Sync>(mut self, service: T) -> Self {
        self.register(service);
        self
    }
}

// Usage - still optional
let registry = ServiceRegistry::new()
    .with(CustomModel::new())      // Override model
    .with(StandardValidator::new()); // Use default validator

let agent = Agent::from_registry(&registry)
    .unwrap_or_else(|| Agent::with_defaults());
```

## Factory Pattern with Trait Objects

```rust
pub trait ComponentFactory: Send + Sync {
    fn create_model(&self) -> Box<dyn LanguageModel>;
    fn create_validator(&self) -> Box<dyn Validator>;
    fn create_monitor(&self) -> Box<dyn Monitor>;
    fn create_storage(&self) -> Box<dyn Storage>;
}

pub struct DefaultFactory;

impl ComponentFactory for DefaultFactory {
    fn create_model(&self) -> Box<dyn LanguageModel> {
        Box::new(OpenRouterModel::new_from_env())
    }
    
    fn create_validator(&self) -> Box<dyn Validator> {
        Box::new(StandardValidator::safe_defaults())
    }
    
    fn create_monitor(&self) -> Box<dyn Monitor> {
        Box::new(TelemetryMonitor::new_otlp())
    }
    
    fn create_storage(&self) -> Box<dyn Storage> {
        Box::new(InMemoryStorage::with_capacity(1000))
    }
}

// Custom factory with overrides
pub struct CustomFactory {
    base: Box<dyn ComponentFactory>,
    model_override: Option<fn() -> Box<dyn LanguageModel>>,
}

impl ComponentFactory for CustomFactory {
    fn create_model(&self) -> Box<dyn LanguageModel> {
        if let Some(factory) = self.model_override {
            factory()
        } else {
            self.base.create_model()
        }
    }
    
    // Delegate other methods to base...
}
```

## Dependency Injection Scopes

### Application Scope (Singleton)

```rust
pub struct ApplicationContext {
    // Shared across entire application
    model_pool: Arc<ModelPool>,
    storage: Arc<dyn Storage>,
    config: Arc<GlobalConfig>,
}

impl ApplicationContext {
    pub fn singleton() -> &'static Self {
        static INSTANCE: OnceCell<ApplicationContext> = OnceCell::new();
        INSTANCE.get_or_init(|| Self::new())
    }
}
```

### Agent Scope (Per-Agent)

```rust
pub struct AgentContext {
    // Unique to each agent instance
    agent_id: AgentId,
    model: Box<dyn LanguageModel>,
    validator: Box<dyn Validator>,
    
    // Reference to application context
    app_context: Arc<ApplicationContext>,
}
```

### Request Scope (Per-Request)

```rust
pub struct RequestContext {
    // Specific to single request
    request_id: RequestId,
    trace_context: TraceContext,
    overrides: Option<RequestOverrides>,
    
    // Reference to agent context
    agent_context: Arc<AgentContext>,
}
```

## Testing with Dependency Injection

The DI pattern greatly simplifies testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockDependencies {
        model: MockModel,
        validator: MockValidator,
    }
    
    impl AgentDependencies for MockDependencies {
        type Model = MockModel;
        type Validator = MockValidator;
        // ...
    }
    
    #[tokio::test]
    async fn test_agent_with_mocks() {
        let mut mock_model = MockModel::new();
        mock_model.expect_complete()
            .returning(|_| Ok(Response::new("mocked")));
        
        let deps = MockDependencies {
            model: mock_model,
            validator: MockValidator::permissive(),
        };
        
        let mut agent = Agent::with_deps(deps);
        let result = agent.execute(test_task()).await;
        assert_eq!(result.unwrap().text, "mocked");
    }
}
```

## Configuration-Driven Injection

```rust
#[derive(Deserialize)]
pub struct DependencyConfig {
    #[serde(default = "default_model")]
    pub model: ModelConfig,
    
    #[serde(default = "default_validator")]
    pub validator: ValidatorConfig,
    
    #[serde(default)]
    pub overrides: HashMap<String, ComponentConfig>,
}

impl DependencyConfig {
    pub fn build_dependencies(&self) -> Result<Box<dyn AgentDependencies>> {
        let mut builder = DependencyBuilder::new();
        
        // Apply configuration
        builder = builder
            .with_model(self.model.build()?)
            .with_validator(self.validator.build()?);
        
        // Apply overrides
        for (name, config) in &self.overrides {
            builder = builder.override_component(name, config.build()?)?;
        }
        
        Ok(Box::new(builder.build()))
    }
}
```

## Best Practices

### 1. Default First
Always provide working defaults. Users should be able to use the framework without any configuration.

### 2. Progressive Disclosure
Simple use cases should be simple. Complex customization should be possible but not required.

### 3. Type Safety Where Possible
Prefer compile-time dependency resolution when the types are known. Use dynamic dispatch only when runtime flexibility is truly needed.

### 4. Explicit Over Magic
Make dependency sources clear. Avoid hidden global state or implicit injection.

### 5. Testability
Design with testing in mind. All dependencies should be replaceable with test doubles.

## Implementation Patterns

### The "Override Chain" Pattern

```rust
pub struct OverrideChain<T> {
    items: Vec<Option<T>>,
}

impl<T> OverrideChain<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn with(mut self, item: Option<T>) -> Self {
        self.items.push(item);
        self
    }
    
    pub fn resolve(self) -> Option<T> {
        // Return first Some value
        self.items.into_iter().flatten().next()
    }
    
    pub fn resolve_or(self, default: T) -> T {
        self.resolve().unwrap_or(default)
    }
}

// Usage
let model = OverrideChain::new()
    .with(request_override)  // Highest priority
    .with(agent_override)    // Medium priority
    .with(global_config)     // Lowest priority
    .resolve_or(default_model());
```

### The "Capability Provider" Pattern

```rust
pub trait CapabilityProvider {
    fn capabilities(&self) -> &[Capability];
    fn provide(&self, cap: &Capability) -> Option<Box<dyn Any>>;
}

pub struct Agent {
    providers: Vec<Box<dyn CapabilityProvider>>,
}

impl Agent {
    pub fn with_capability<T: Any>(&self) -> Option<&T> {
        let cap = Capability::of::<T>();
        for provider in &self.providers {
            if provider.capabilities().contains(&cap) {
                if let Some(any) = provider.provide(&cap) {
                    return any.downcast_ref::<T>();
                }
            }
        }
        None
    }
}
```

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/configuration_strategy.md] - integrates - Configuration-based injection
  - [elements/rust_patterns.md] - uses - Rust-specific patterns
  - [foundation/principles.md] - embodies - Core design principles

## Navigation Guidance
- **Access Context:** Reference when designing component relationships and customization points
- **Common Next Steps:** Review configuration strategy or protocol abstractions
- **Related Tasks:** Component design, testing strategy, API design
- **Update Patterns:** Update when adding new injection patterns or scopes

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial design of dependency injection philosophy with freedom-without-obligation principle