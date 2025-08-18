# Patinox Source Code

This directory contains the Rust implementation of the Patinox AI agent framework.

## Structure

The source code is organized as a Rust workspace with multiple crates:

```
src/
├── Cargo.toml              # Workspace configuration
├── patinox-core/          # Core traits and types
├── patinox-agent/         # Agent implementation
├── patinox-validation/    # Validation pipeline
├── patinox-monitor/       # Monitoring layer
├── patinox-runtime/       # Execution runtime
├── patinox-telemetry/     # OpenTelemetry integration
├── patinox-storage/       # Vector DB integration
├── patinox-meta/          # Meta-layer analysis
├── patinox-evolution/     # Git-based evolution
├── patinox-bindings/      # Language bindings
└── examples/              # Example usage
```

## Development Status

🚧 **Early Development** - Source code will be added as the project progresses through its implementation phases. See the [roadmap](../context-network/planning/roadmap.md) for the current development phase.

## Getting Started

Once development begins, you'll be able to:

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

## Contributing

All architectural decisions and planning happen in the [context network](../context-network/). Please review the documentation there before contributing code.

## License

[License information will be added]