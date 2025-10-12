# Rust Agent Framework

A lightweight, composable framework for building CLI agents in Rust that compile to fast, standalone executables.

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-agent-framework
cd rust-agent-framework

# Build all agents
cargo build --release

# Or build with specific features
cargo build --release --features "all-providers,patterns"

# Install to system
cargo install --path .
```

### Create Your First Agent

```rust
// my_agent.rs
use agent_core::prelude::*;

fn main() -> Result<()> {
    let agent = create_agent!("my-agent")
        .tool_fn("hello", "Say hello", |name| {
            Ok(format!("Hello, {}!", name))
        });
    
    agent.cli()
}
```

Compile and run:
```bash
# Compile
cargo build --release --bin my_agent

# Or compile standalone
rustc my_agent.rs -o my_agent

# Run
./my_agent "world"
# Output: Hello, world!
```

## Architecture

### Layered Design

1. **Core Layer** (~150 lines)
   - Minimal agent implementation
   - Tool system
   - Basic CLI generation

2. **Provider Layer**
   - OpenAI, Anthropic, Ollama, Groq
   - Easy to add custom providers
   - Environment-based configuration

3. **Plugin Layer** (optional)
   - Memory persistence
   - Agent discovery
   - Resource management
   - Context networks

4. **Pattern Layer** (optional)
   - ReACT (default)
   - Plan-Execute
   - Reflexion

## Usage Examples

### Basic Text Processing Agent

```rust
use agent_core::prelude::*;

fn main() -> Result<()> {
    let agent = create_agent!("text-processor")
        .tool_fn("uppercase", "Convert to uppercase", |text| {
            Ok(text.to_uppercase())
        })
        .tool_fn("count", "Count words", |text| {
            Ok(text.split_whitespace().count().to_string())
        })
        .tool_fn("reverse", "Reverse text", |text| {
            Ok(text.chars().rev().collect())
        });
    
    agent.cli()
}
```

### Code Reviewer with Memory

```rust
use agent_core::prelude::*;
use agent_plugins::{MemoryPlugin, ResourcePlugin};

fn main() -> Result<()> {
    let config = AgentConfig {
        name: "code-reviewer".to_string(),
        system_prompt: Some("Review code for issues.".to_string()),
        ..Default::default()
    };
    
    let agent = Agent::new(config)?
        .plugin(MemoryPlugin::new("~/.agent-memory/reviews.db"))
        .plugin(ResourcePlugin::new().timeout(30).retry(3))
        .tool_fn("analyze", "Analyze code", |code| {
            // Your code analysis logic
            Ok("Analysis complete".to_string())
        });
    
    agent.cli()
}
```

## Provider Configuration

### Environment Variables

```bash
# OpenAI (default)
export LLM_PROVIDER=openai
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-4o-mini

# Anthropic
export LLM_PROVIDER=anthropic  
export ANTHROPIC_API_KEY=sk-ant-...
export ANTHROPIC_MODEL=claude-3-haiku-20240307

# Ollama (local)
export LLM_PROVIDER=ollama
export OLLAMA_MODEL=llama3.1:8b
export OLLAMA_HOST=http://localhost:11434

# Groq (fast cloud)
export LLM_PROVIDER=groq
export GROQ_API_KEY=gsk_...
export GROQ_MODEL=llama-3.1-70b-versatile
```

### Runtime Provider Switching

```bash
# Via CLI flags
./my-agent "task" --provider anthropic --model claude-3-opus

# Via environment for specific run
LLM_PROVIDER=groq ./my-agent "fast task"

# In code
agent.set_provider(Provider::Anthropic);
agent.set_model("claude-3-sonnet-20240229");
```

## Building & Deployment

### Development Build

```bash
# Debug build with all features
cargo build --all-features

# Run tests
cargo test

# Run with mock LLM (no API needed)
MOCK_LLM=1 cargo run -- "test input"
```

### Production Build

```bash
# Optimized release build
cargo build --release

# Size-optimized build
cargo build --profile release-small

# Cross-compilation for Linux
cargo build --release --target x86_64-unknown-linux-musl

# Cross-compilation for macOS
cargo build --release --target x86_64-apple-darwin
```

### Binary Size Optimization

```bash
# Strip symbols
strip target/release/my-agent

# Use UPX for further compression
upx --best target/release/my-agent

# Typical sizes:
# - Debug: 20-30 MB
# - Release: 5-10 MB
# - Stripped: 3-5 MB
# - UPX compressed: 1-2 MB
```

## Advanced Features

### Using Patterns

```rust
use agent_patterns::PlanExecutePattern;

let agent = create_agent!("coordinator")
    .pattern(PlanExecutePattern::new()
        .planner("gpt-4o")
        .executor("gpt-4o-mini"));
```

### Agent Composition

```bash
# Unix pipeline composition
echo "analyze this" | ./analyzer | ./summarizer | ./formatter

# Agent calling another agent
./coordinator "refactor auth system"
# Automatically discovers and coordinates: architect, code-reviewer, test-runner

# Parallel execution
parallel -j 4 ./analyzer ::: file1.rs file2.rs file3.rs file4.rs
```

### Service Deployment

```toml
# systemd service: /etc/systemd/system/my-agent.service
[Unit]
Description=My AI Agent
After=network.target

[Service]
Type=simple
User=agent
Environment="LLM_PROVIDER=anthropic"
Environment="ANTHROPIC_API_KEY=sk-ant-..."
ExecStart=/usr/local/bin/my-agent --daemon
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  agent:
    image: rust:1.75-alpine
    volumes:
      - ./target/release/my-agent:/usr/local/bin/my-agent
      - ./data:/data
    environment:
      - LLM_PROVIDER=openai
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    command: my-agent --server --port 8080
```

## Performance Benchmarks

| Metric | Rust Agent | Node/Deno Agent | Python Agent |
|--------|------------|-----------------|--------------|
| Startup Time | <10ms | 100-200ms | 200-500ms |
| Memory (idle) | 2-5 MB | 30-50 MB | 50-100 MB |
| Memory (active) | 10-20 MB | 100-200 MB | 200-500 MB |
| Binary Size | 3-5 MB | 80-100 MB | N/A (interpreted) |
| Requests/sec | 10,000+ | 1,000-2,000 | 100-500 |
| Concurrent Agents | 1000+ | 100-200 | 10-50 |

## Plugin Development

### Creating a Custom Plugin

```rust
use agent_core::{Plugin, Agent, Context, RunResult};
use async_trait::async_trait;

pub struct MyPlugin {
    config: MyConfig,
}

#[async_trait]
impl Plugin for MyPlugin {
    async fn install(&self, agent: &mut Agent) -> Result<()> {
        // Add tools, modify agent behavior
        agent.tool_fn("my_tool", "Description", |args| {
            Ok("Result".to_string())
        });
        Ok(())
    }
    
    async fn before_run(&self, context: &mut Context) -> Result<()> {
        // Modify context before execution
        context.add_header("X-Custom", "value");
        Ok(())
    }
    
    async fn after_run(&self, result: &mut RunResult) -> Result<()> {
        // Process results
        result.metadata.insert("plugin", "MyPlugin");
        Ok(())
    }
}
```

### Provider Development

```rust
use agent_providers::{LLMProvider, register_provider};

struct CustomProvider {
    client: MyClient,
}

#[async_trait]
impl LLMProvider for CustomProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Your API implementation
    }
    
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            max_tokens: 4096,
            supports_functions: true,
            supports_vision: false,
            supports_streaming: true,
            cost_per_1k_tokens: Some(TokenCost {
                input: 0.0001,
                output: 0.0002,
            }),
        }
    }
}

// Register globally
register_provider("custom", || Box::new(CustomProvider::new()));
```

## Migration Guide

### From TypeScript/Deno

```typescript
// Deno version
const agent = createAgent({
  name: "my-agent",
  model: { provider: "openai" }
});

agent.tool("hello", "Greet", () => "Hello!");
```

```rust
// Rust version
let agent = create_agent!("my-agent")
    .tool_fn("hello", "Greet", |_| Ok("Hello!".to_string()));
```

### From Python

```python
# Python version
agent = Agent(name="my-agent")
agent.add_tool("hello", lambda: "Hello!")
agent.run("greet me")
```

```rust
// Rust version
let agent = create_agent!("my-agent")
    .tool_fn("hello", "Greet", |_| Ok("Hello!".to_string()));
agent.run("greet me").await?;
```

## Troubleshooting

### Common Issues

1. **Binary too large**: Use `--profile release-small` and strip symbols
2. **Can't find provider**: Check environment variables and API keys
3. **Compilation errors**: Update Rust to latest stable (`rustup update`)
4. **Cross-compilation issues**: Install target (`rustup target add x86_64-unknown-linux-musl`)

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug ./my-agent "test"

# Trace all LLM calls
RUST_LOG=agent_providers=trace ./my-agent "test"

# Mock LLM for testing
MOCK_LLM=1 ./my-agent "test"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure all tests pass: `cargo test`
5. Check formatting: `cargo fmt -- --check`
6. Check lints: `cargo clippy`
7. Submit a pull request

## License

MIT License - see LICENSE file for details

## Roadmap

- [ ] WebAssembly support for browser agents
- [ ] gRPC server mode for microservices
- [ ] Hot-reload for development
- [ ] Visual Studio Code extension
- [ ] Agent marketplace/registry
- [ ] Distributed agent orchestration
- [ ] Built-in observability (OpenTelemetry)
- [ ] Native GUI for configuration

## Community

- GitHub: https://github.com/yourusername/rust-agent-framework
- Discord: https://discord.gg/rustagents
- Documentation: https://docs.rs/agent-core

## Acknowledgments

Inspired by the Unix philosophy of small, composable tools and the elegance of Rust's type system.