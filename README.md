# Patinox

**Minimal to sophisticated AI agents in Rust**

A layered agent framework that starts simple and grows with your needs—from ~150 line CLI tools to enterprise-grade orchestration with embedded monitoring.

## Quick Start

```rust
use patinox::*;

fn main() {
    create_agent("hello")
        .tool_fn("greet", "Say hello", |name| {
            Ok(format!("Hello, {}!", name))
        })
        .run_cli()
}
```

```bash
cargo build --release
./target/release/hello "world"
# Output: Hello, world!
```

## Architecture Layers

Patinox uses progressive enhancement—start simple, add sophistication only when needed:

### Layer 1: Minimal Agent (Current Focus)
```rust
// ~150 lines of core functionality
use patinox::*;

fn main() {
    create_agent("processor")
        .tool_fn("uppercase", "Convert to uppercase", |text| {
            Ok(text.to_uppercase())
        })
        .tool_fn("count", "Count words", |text| {
            Ok(text.split_whitespace().count().to_string())
        })
        .provider(Provider::Anthropic)
        .run_cli()
}
```

**Week 1 Goal**: Working agent you can actually use

### Layer 2: Plugin Enhancements (Coming Soon)
```rust
use patinox::{*, plugins::*};

let agent = create_agent("reviewer")
    .plugin(MemoryPlugin::new("~/.patinox/memory"))
    .plugin(DiscoveryPlugin::new())
    .tools(/* ... */);
```

Add capabilities as you need them:
- Memory persistence
- Agent discovery
- Resource management

### Layer 3: Reasoning Patterns (Roadmap)
```rust
use patinox::patterns::*;

let agent = create_agent("coordinator")
    .pattern(PlanExecutePattern::new())
    .tools(/* ... */);
```

Sophisticated reasoning when simple isn't enough:
- Plan-Execute
- Reflexion
- Multi-step orchestration

### Layer 4: Enterprise Features (Import from V1)
```rust
use patinox::enterprise::*;

let agent = create_agent("production")
    .validation(TowerMiddleware::new()
        .layer(AntiJailbreakLayer)
        .layer(RateLimitLayer))
    .monitoring(MapekMonitor::new())
    .tools(/* ... */);
```

Production-grade features when validated:
- MAPE-K embedded monitoring
- Tower validation middleware
- Type-safe state machines
- Git-based evolution
- Full observability

## Philosophy

**Build the trail, not just the summit.**

Patinox starts minimal and grows through real usage, not anticipated needs. Enterprise features exist as validated imports, not premature abstractions.

### What Makes Patinox Different

1. **Progressive Enhancement**: Start with ~150 lines, add layers only when needed
2. **Rust Performance**: Native speed with compile-time safety
3. **Real Usage Validation**: Features emerge from pain points, not predictions
4. **Clear Graduation Path**: Simple → Sophisticated is explicit and documented

## Current Status

🚧 **V2 Implementation - Week 1: Minimal Core**

- ✅ Strategic reset complete (see [V2 Decision](./context-network/decisions/v2_strategic_reset.md))
- ✅ V1 research archived for enterprise tier import
- 🚧 Building ~150 line working agent
- ⏳ First real usage example

**What's Working**: Nothing yet! We just reset to minimal-first.
**Next Milestone**: Working agent by end of Week 1

## V1 Research

Patinox V1 (sophisticated-first) provided valuable research into enterprise agent frameworks. That work is preserved at:
- Branch: `archive/patinox-v1-sophisticated-first`
- Tag: `v1-research-phase`
- Documentation: `context-network/archive/v1-research/`

V1 code will be imported as Layer 4 (Enterprise Features) when Layer 1-3 validate the need through real usage.

## Installation

```bash
# Clone the repository
git clone https://github.com/jwynia/patinox.git
cd patinox

# Build (once minimal core is complete)
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
```

## Roadmap

### Week 1: Minimal Agent
- [ ] ~150 line core implementation
- [ ] Provider abstraction (OpenAI/Anthropic)
- [ ] Basic tool system
- [ ] CLI interface
- [ ] First working example

### Week 2-3: Plugin Layer
- [ ] Memory plugin
- [ ] Discovery plugin
- [ ] Resource management
- [ ] Based on Week 1 usage discoveries

### Week 4+: Pattern & Enterprise Layers
- [ ] Reasoning patterns when needed
- [ ] Import V1 validation logic
- [ ] Import V1 monitoring system
- [ ] Enterprise tier as validated

## Examples

Coming soon! We're building the minimal core first, then examples will emerge from real usage.

## Contributing

Patinox follows a minimal-first philosophy. Contributions should:
1. Start with simple use cases
2. Add complexity only when pain is felt
3. Document real-world usage patterns
4. Follow progressive enhancement layers

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details (coming soon).

## Context Network

This project uses a context network for planning and coordination. See [context-network/discovery.md](./context-network/discovery.md) for navigation.

**Note**: V1 planning documents are archived. Current planning focuses on minimal-first approach.

## Why Rust?

- **Performance**: Native speed without GC overhead
- **Safety**: Compile-time prevention of entire error classes
- **Zero-cost abstractions**: Sophistication without runtime penalty
- **WebAssembly**: Universal deployment target
- **Type system**: Progressive enhancement of safety guarantees

## License

MIT OR Apache-2.0

## Acknowledgments

Inspired by:
- Unix philosophy of composable tools
- Rust agent framework research (see inbox documents)
- Mastra's problem space understanding
- Anthropic's embedded monitoring research

Built on excellent Rust libraries:
- async-openai for LLM integration
- Tokio for async runtime
- Tower for middleware (Layer 4)
- OpenTelemetry for observability (Layer 4)

---

**Status**: V2 Implementation Active | V1 Research Archived | Focus: Minimal Core First
