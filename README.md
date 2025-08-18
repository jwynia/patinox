# Patinox

A ground-up reimagining of AI agent orchestration that prioritizes safety, observability, and systematic evolution through compile-time guarantees and embedded monitoring.

## Vision

Build an AI agent framework in Rust that treats monitoring and validation as first-class architectural concerns rather than afterthoughts. By leveraging Rust's type system and ownership model, we can create agent systems that are simultaneously more reliable and more capable of self-improvement than existing dynamic implementations.

## Core Philosophy

**Safety through compilation, not convention.** Invalid agent states should be unrepresentable. Monitoring should be inescapable. Evolution should be traceable.

This framework addresses the same fundamental problems as TypeScript-based Mastra (tools, workflows, memory, telemetry, evaluation, scoring) but reimagines the solutions through Rust's lens of zero-cost abstractions and compile-time guarantees.

## Key Innovations

### 1. Embedded Monitoring Architecture
- **Synchronous validators** that act as compile-time-configured quality gates
- **Asynchronous analyzers** that identify patterns and propose improvements
- Both LLM-based monitors (anti-jailbreak, hallucination detection) and traditional ML (Bayesian classifiers, rule engines)
- Validators can divert, retry, or modify agent execution in real-time

### 2. Git-Based Evolution Loop
Rather than runtime mutation (which can go "off the rails"), the framework uses a traceable evolution pattern:
- One instance executes agents and logs telemetry
- Another instance analyzes patterns and generates PRs with improvements
- Changes are reviewed, merged, and deployed through standard CI/CD
- Every evolution step is auditable and reversible

### 3. Compile-Time Workflow Validation
Using Rust's type system to make invalid states unrepresentable:
- Typestate patterns ensure agents can only transition through valid states
- Phantom types guarantee complete configuration before execution
- Trait-based composition allows pluggable validators without modifying core code

### 4. Native Performance with Universal Deployment
- Zero-copy integration with Rust-based vector databases (Qdrant, LanceDB)
- WebAssembly compilation for edge deployment
- Native bindings for Python/TypeScript migration paths

## Technical Foundation

Built on production-proven Rust libraries:
- **async-openai** for LLM integration (1.1M+ downloads)
- **Rig** for LLM application patterns
- **Tower** middleware for composable validation layers
- **OpenTelemetry** for observability
- **Qdrant/LanceDB** for vector storage
- **Tokio** for async runtime

## Target Use Cases

- Production systems requiring strict safety guarantees
- Multi-agent systems with complex interaction patterns
- Applications needing transparent, auditable AI behavior
- Systems that must evolve and improve over time
- High-performance edge deployments

## Project Status

🛑 **PLANNING PHASE ONLY - NO CODING YET**

**Critical**: We are in the planning and design phase. NO implementation code will be written until an explicit, documented decision is made to begin coding. See [CRITICAL_NO_CODING_YET.md](./context-network/decisions/CRITICAL_NO_CODING_YET.md) for details.

**Current Focus**: 
- Finalizing architectural decisions
- Validating design patterns
- Building complete understanding
- Documenting all plans thoroughly

**Status**: Synthesizing research from 380+ sources across Rust ecosystem, AI agent patterns, and production monitoring systems.

## Development Workflow

This project uses a context network for all planning, research, and coordination. See the [context network documentation](./context-network/discovery.md) for navigation.

### Getting Started

1. **Review the context network** at `./context-network/` for project architecture and decisions
2. **Check the roadmap** at `./context-network/planning/roadmap.md` for current phase
3. **Source code** will be in `/src/` as the project progresses

### Contributing

All planning and conceptual work happens in the context network. Implementation follows the patterns and decisions documented there.

## Why Rust?

Rust uniquely enables:
- **Memory safety** without garbage collection overhead
- **Compile-time validation** of agent workflows
- **Zero-cost abstractions** for monitoring layers
- **Fearless concurrency** for parallel agent execution
- **WebAssembly target** for universal deployment

## Next Steps

1. Define core trait abstractions for agents, tools, and validators
2. Implement proof-of-concept with synchronous validation pipeline
3. Add asynchronous monitoring and telemetry collection
4. Build meta-layer for analyzing telemetry and proposing improvements
5. Create development tools and documentation
6. Release initial version with Python/TypeScript bindings

---

*Inspired by [Mastra](https://mastra.dev)'s problem space and [Anthropic's research](https://www.anthropic.com/engineering/multi-agent-research-system) on embedded monitoring, Patinox explores how Rust's unique capabilities can advance the state of the art in AI agent development.*