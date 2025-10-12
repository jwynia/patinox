# Configuration Strategy

## Purpose
This document defines the comprehensive configuration strategy for Patinox, including cascading configuration, environment-based settings, and runtime overrides.

## Classification
- **Domain:** Technical Architecture
- **Stability:** Semi-stable
- **Abstraction:** Detailed
- **Confidence:** High

## Content

### Configuration Philosophy

1. **Convention over Configuration**: Sensible defaults that work without any configuration
2. **Progressive Disclosure**: Simple cases are simple, complex cases are possible
3. **Cascading Overrides**: Settings cascade from global → agent → request level
4. **Environment-First**: Production credentials come from environment, not files
5. **Type-Safe**: Configuration is validated at compile time where possible

### Configuration Hierarchy

```
┌─────────────────────────────────────┐
│     Request-Level Config            │ ← Highest Priority
│  (Runtime method parameters)        │
├─────────────────────────────────────┤
│     Agent-Level Config              │
│  (Agent builder settings)           │
├─────────────────────────────────────┤
│     Environment Variables           │
│  (PATINOX_* variables)             │
├─────────────────────────────────────┤
│     Configuration Files             │
│  (patinox.toml, patinox.yaml)      │
├─────────────────────────────────────┤
│     Framework Defaults              │ ← Lowest Priority
│  (Hardcoded sensible defaults)     │
└─────────────────────────────────────┘
```

### Configuration Sources

#### 1. Framework Defaults

Built into the code, these provide a zero-configuration starting point:

```rust
impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                provider: Provider::OpenRouter,
                model: ModelId::new("anthropic/claude-3-sonnet"),
                temperature: 0.7,
                max_tokens: 2048,
                timeout: Duration::from_secs(30),
            },
            monitoring: MonitoringConfig {
                enabled: true,
                sample_rate: 0.1,
                export_endpoint: None,  // Disabled by default
            },
            validation: ValidationConfig {
                enabled: true,
                anti_jailbreak: true,
                hallucination_detection: false,  // Opt-in
                rate_limit: Some(60),  // 60 requests per minute
            },
            storage: StorageConfig {
                provider: StorageProvider::InMemory,
                max_memory_mb: 100,
            },
        }
    }
}
```

#### 2. Configuration Files

TOML format (preferred) or YAML, searched in order:
1. `./.patinox/config.toml` (local dotfolder - highest priority)
2. `./patinox.toml` (current directory)
3. `${WORKSPACE_FOLDER}/.patinox/config.toml` (workspace dotfolder for devcontainers)
4. `~/.config/patinox/config.toml` (user config - avoided in containers)
5. `/etc/patinox/config.toml` (system config - lowest priority)

**Dotfolder Philosophy**: Local `.patinox` folders provide persistent, project-specific configuration that:
- Survives container rebuilds
- Can be gitignored for local overrides
- Provides clear configuration locality
- Avoids home directory pollution in containers

Example `patinox.toml`:
```toml
[model]
provider = "openrouter"
default_model = "anthropic/claude-3-opus"
temperature = 0.7
max_tokens = 4096

# Model-specific overrides
[model.overrides."gpt-4"]
temperature = 0.5
max_tokens = 8192

[model.overrides."claude-3-haiku"]
temperature = 0.9
max_tokens = 2048

[providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
# API key from environment: OPENROUTER_API_KEY

[providers.openai]
organization = "org-123"
# API key from environment: OPENAI_API_KEY

[monitoring]
enabled = true
sample_rate = 1.0  # Sample everything in dev
export_endpoint = "http://localhost:4317"
service_name = "patinox-dev"

[validation]
enabled = true
anti_jailbreak = true
hallucination_detection = true

[[validation.custom_validators]]
name = "profanity_filter"
enabled = true
config = { level = "strict" }

[[validation.custom_validators]]
name = "pii_detector"
enabled = true
config = { patterns = ["ssn", "credit_card"] }

[storage]
provider = "qdrant"
url = "http://localhost:6333"
collection = "patinox_memory"

[rate_limits]
requests_per_minute = 60
tokens_per_minute = 100000
concurrent_requests = 10

[retry]
max_attempts = 3
initial_delay_ms = 1000
max_delay_ms = 10000
exponential_base = 2
```

#### 3. Environment Variables

Environment variables override file configuration. All variables prefixed with `PATINOX_`:

```bash
# Model configuration
PATINOX_MODEL_PROVIDER=openrouter
PATINOX_MODEL_DEFAULT=claude-3-opus
PATINOX_MODEL_TEMPERATURE=0.7
PATINOX_MODEL_MAX_TOKENS=4096
PATINOX_MODEL_TIMEOUT=30s

# Provider API keys (special handling - not prefixed)
OPENROUTER_API_KEY=sk-or-...
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Monitoring
PATINOX_MONITORING_ENABLED=true
PATINOX_MONITORING_ENDPOINT=http://otel-collector:4317
PATINOX_MONITORING_SAMPLE_RATE=0.1

# Validation
PATINOX_VALIDATION_ENABLED=true
PATINOX_VALIDATION_ANTI_JAILBREAK=true
PATINOX_VALIDATION_HALLUCINATION=false

# Storage
PATINOX_STORAGE_PROVIDER=qdrant
PATINOX_STORAGE_URL=http://qdrant:6333

# Rate limits
PATINOX_RATE_LIMIT_RPM=60
PATINOX_RATE_LIMIT_TPM=100000
```

#### 4. Agent-Level Configuration

Set when creating an agent:

```rust
let agent = Agent::builder()
    .with_model("gpt-4-turbo")
    .with_temperature(0.5)
    .with_max_tokens(8192)
    .with_timeout(Duration::from_secs(60))
    .with_validator(CustomValidator::new())
    .with_monitor(PerformanceMonitor::new())
    .build()?;
```

#### 5. Request-Level Configuration

Override per request:

```rust
let response = agent
    .complete("Explain quantum computing")
    .with_model("claude-3-opus")  // Override for this request
    .with_temperature(0.3)  // More focused response
    .with_max_tokens(4096)  // Longer response
    .with_timeout(Duration::from_secs(120))  // More time
    .await?;
```

### Configuration Loading Implementation

```rust
pub struct ConfigLoader {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            sources: vec![
                Box::new(DefaultsSource),
                Box::new(FileSource::new()),
                Box::new(EnvSource),
            ],
        }
    }
    
    pub async fn load(&self) -> Result<GlobalConfig, ConfigError> {
        let mut config = GlobalConfig::default();
        
        // Apply each source in order
        for source in &self.sources {
            if let Some(partial) = source.load().await? {
                config.merge(partial)?;
            }
        }
        
        // Validate the final configuration
        config.validate()?;
        
        // Resolve special values (e.g., ${VAR} references)
        config.resolve_variables()?;
        
        Ok(config)
    }
}

trait ConfigSource: Send + Sync {
    async fn load(&self) -> Result<Option<PartialConfig>, ConfigError>;
}
```

### Configuration Validation

```rust
impl GlobalConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate model configuration
        if self.model.temperature < 0.0 || self.model.temperature > 2.0 {
            return Err(ConfigError::InvalidValue {
                field: "model.temperature",
                reason: "Must be between 0.0 and 2.0",
            });
        }
        
        if self.model.max_tokens == 0 {
            return Err(ConfigError::InvalidValue {
                field: "model.max_tokens",
                reason: "Must be greater than 0",
            });
        }
        
        // Validate rate limits
        if let Some(rpm) = self.rate_limits.requests_per_minute {
            if rpm == 0 {
                return Err(ConfigError::InvalidValue {
                    field: "rate_limits.requests_per_minute",
                    reason: "Must be greater than 0 or None",
                });
            }
        }
        
        // Validate monitoring
        if self.monitoring.sample_rate < 0.0 || self.monitoring.sample_rate > 1.0 {
            return Err(ConfigError::InvalidValue {
                field: "monitoring.sample_rate",
                reason: "Must be between 0.0 and 1.0",
            });
        }
        
        Ok(())
    }
}
```

### Secret Management

Secrets are handled specially:

```rust
pub enum SecretValue {
    /// Direct value (not recommended for production)
    Direct(String),
    /// Environment variable reference
    EnvVar(String),
    /// File path containing the secret
    File(PathBuf),
    /// External secret manager (HashiCorp Vault, AWS Secrets Manager, etc.)
    External {
        provider: SecretProvider,
        key: String,
    },
}

impl SecretValue {
    pub async fn resolve(&self) -> Result<SecretString, ConfigError> {
        match self {
            Self::Direct(value) => {
                warn!("Using direct secret value - not recommended for production");
                Ok(SecretString::new(value.clone()))
            }
            Self::EnvVar(name) => {
                std::env::var(name)
                    .map(SecretString::new)
                    .map_err(|_| ConfigError::MissingSecret(name.clone()))
            }
            Self::File(path) => {
                tokio::fs::read_to_string(path)
                    .await
                    .map(|s| SecretString::new(s.trim().to_string()))
                    .map_err(|e| ConfigError::SecretFileError(path.clone(), e))
            }
            Self::External { provider, key } => {
                provider.fetch_secret(key).await
            }
        }
    }
}
```

### Dynamic Configuration Updates

Support for configuration hot-reloading:

```rust
pub struct ConfigWatcher {
    config: Arc<RwLock<GlobalConfig>>,
    watcher: FileWatcher,
}

impl ConfigWatcher {
    pub async fn watch(&self) -> Result<(), ConfigError> {
        let config = self.config.clone();
        
        self.watcher.watch("patinox.toml", move |event| {
            if let Event::Modified = event {
                info!("Configuration file changed, reloading...");
                
                match ConfigLoader::new().load() {
                    Ok(new_config) => {
                        let mut config_write = config.write().await;
                        *config_write = new_config;
                        info!("Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("Failed to reload configuration: {}", e);
                        // Keep existing configuration on error
                    }
                }
            }
        }).await?;
        
        Ok(())
    }
}
```

### Configuration Schema Export

Generate configuration schema for validation and IDE support:

```rust
impl GlobalConfig {
    pub fn json_schema() -> serde_json::Value {
        // Generate JSON Schema for configuration validation
        schemars::schema_for!(GlobalConfig)
    }
    
    pub fn write_schema(path: &Path) -> Result<(), ConfigError> {
        let schema = Self::json_schema();
        let json = serde_json::to_string_pretty(&schema)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn generate_template() -> String {
        // Generate a template configuration file with all options
        toml::to_string_pretty(&Self::example()).unwrap()
    }
}
```

### Devcontainer-Aware Configuration

Special handling for development containers:

```rust
pub struct DevcontainerConfig {
    /// Detect if running in a devcontainer
    pub fn detect_devcontainer() -> bool {
        // Check for devcontainer environment markers
        env::var("REMOTE_CONTAINERS").is_ok() ||
        env::var("CODESPACES").is_ok() ||
        env::var("DEVCONTAINER").is_ok() ||
        Path::new("/.dockerenv").exists()
    }
    
    /// Get appropriate config paths for container environment
    pub fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Local dotfolder (always first)
        paths.push(PathBuf::from("./.patinox"));
        
        // Workspace folder in devcontainer
        if let Ok(workspace) = env::var("WORKSPACE_FOLDER") {
            paths.push(PathBuf::from(workspace).join(".patinox"));
        }
        
        // Only use home directory if not in container
        if !Self::detect_devcontainer() {
            if let Some(home) = dirs::config_dir() {
                paths.push(home.join("patinox"));
            }
        }
        
        // System config (always last)
        paths.push(PathBuf::from("/etc/patinox"));
        
        paths
    }
    
    /// Mount-aware file resolution
    pub fn resolve_path(path: &Path) -> PathBuf {
        if Self::detect_devcontainer() {
            // Resolve relative to workspace
            if let Ok(workspace) = env::var("WORKSPACE_FOLDER") {
                let workspace_path = PathBuf::from(workspace);
                if path.is_relative() {
                    return workspace_path.join(path);
                }
            }
        }
        
        path.to_path_buf()
    }
}
```

### Dotfolder Structure

The `.patinox` folder provides organized local configuration:

```
.patinox/
├── config.toml          # Main configuration
├── agents/              # Agent-specific configs
│   ├── researcher.toml
│   └── analyzer.toml
├── credentials/         # Local credentials (gitignored)
│   ├── api_keys.env
│   └── certificates/
├── cache/              # Local cache data
│   ├── models/
│   └── tools/
├── state/              # Persistent state
│   ├── checkpoints/
│   └── history/
└── overrides/          # Environment-specific overrides
    ├── development.toml
    ├── testing.toml
    └── production.toml
```

### Development vs Production

Different configuration profiles:

```rust
#[derive(Clone, Debug)]
pub enum ConfigProfile {
    Development,
    Testing,
    Staging,
    Production,
}

impl ConfigProfile {
    pub fn from_env() -> Self {
        match std::env::var("PATINOX_ENV").as_deref() {
            Ok("production") | Ok("prod") => Self::Production,
            Ok("staging") | Ok("stage") => Self::Staging,
            Ok("testing") | Ok("test") => Self::Testing,
            _ => Self::Development,
        }
    }
    
    pub fn apply_defaults(&self, config: &mut GlobalConfig) {
        match self {
            Self::Development => {
                config.monitoring.sample_rate = 1.0;  // Sample everything
                config.validation.enabled = false;  // Faster iteration
                config.model.timeout = Duration::from_secs(120);  // More lenient
            }
            Self::Testing => {
                config.model.provider = Provider::Mock;  // Use mock provider
                config.storage.provider = StorageProvider::InMemory;
            }
            Self::Production => {
                config.monitoring.sample_rate = 0.1;  // Sample 10%
                config.validation.enabled = true;  // All validators on
                config.retry.max_attempts = 5;  // More retries
            }
            _ => {}
        }
    }
}
```

### Configuration CLI

Command-line tools for configuration management:

```bash
# Validate configuration
patinox config validate

# Show effective configuration (with all sources merged)
patinox config show

# Generate configuration template
patinox config init

# Test configuration loading
patinox config test

# Export JSON schema
patinox config schema > patinox-config.schema.json
```

## Benefits

1. **Flexibility**: Configure at any level of granularity
2. **Security**: Secrets never in code or config files
3. **Simplicity**: Zero config works out of the box
4. **Power**: Full control when needed
5. **Safety**: Validation prevents invalid configurations
6. **Observability**: Configuration changes are logged
7. **Container-Native**: Works seamlessly in devcontainers
8. **Local Persistence**: Dotfolders survive container rebuilds
9. **No Home Pollution**: Avoids cluttering user home in containers

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/model_provider_abstraction.md] - configures - Model settings
  - [elements/monitoring_strategy.md] - configures - Monitoring settings
  - [foundation/principles.md] - follows - Configuration philosophy

## Navigation Guidance
- **Access Context:** Reference when implementing configuration loading or validation
- **Common Next Steps:** Review model provider abstraction or monitoring configuration
- **Related Tasks:** Environment setup, secret management, deployment configuration
- **Update Patterns:** Update when adding new configuration options or sources

## Metadata
- **Created:** 2025-01-17
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-17: Created comprehensive configuration strategy with cascading overrides
- 2025-01-18: Enhanced with devcontainer awareness and dotfolder patterns for local persistence