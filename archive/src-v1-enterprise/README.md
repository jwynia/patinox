# Patinox Source Code

This directory contains the Rust implementation of the Patinox AI agent framework.

## Structure

The source code is organized as a Rust workspace with multiple crates:

```
patinox/
├── Cargo.toml              # Workspace configuration
├── src/                    # Main library (foundational utilities)
│   └── lib.rs              # Core library code
├── patinox-core/          # Core traits and types (future)
├── patinox-agent/         # Agent implementation (future)
├── patinox-validation/    # Validation pipeline (future)
├── patinox-monitor/       # Monitoring layer (future)
├── patinox-runtime/       # Execution runtime (future)
├── patinox-telemetry/     # OpenTelemetry integration (future)
├── patinox-storage/       # Vector DB integration (future)
├── patinox-meta/          # Meta-layer analysis (future)
├── patinox-evolution/     # Git-based evolution (future)
├── patinox-bindings/      # Language bindings (future)
├── examples/              # Example usage (future)
└── tests/                 # Integration tests
```

## Development Status

🚧 **Foundation Phase** - Currently implementing core foundational utilities following a test-driven, utility-first approach:

- ✅ Project structure and tooling setup complete
- 🚧 Next: Core error types and trait definitions  
- ⏳ Future: Individual specialized crates

See the [roadmap](../context-network/planning/roadmap.md) for the complete development plan.

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