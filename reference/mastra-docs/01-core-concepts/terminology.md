# Terminology

> Domain-specific terms and their meanings in Mastra

## Overview

This glossary defines key terms used throughout Mastra documentation and codebase. Understanding these terms is essential for effective use of the framework.

## Core Terms

### Agent
An AI system that uses language models to choose sequences of actions. Agents combine LLMs with tools, memory, and structured reasoning to accomplish tasks.

### Tool
A typed function that can be executed by agents or workflows. Tools extend capabilities by providing access to external data, APIs, or computations.

### Workflow
A durable, graph-based state machine that orchestrates multi-step processes. Workflows support branching, loops, suspend/resume, and error handling.

### Step
An individual unit of work within a workflow. Each step has typed inputs/outputs and can be synchronous or asynchronous.

### Memory
A system for storing and retrieving conversation history and context. Includes thread management, message persistence, and semantic recall.

### Storage
The persistence layer abstraction that provides a unified interface for different backend stores (PostgreSQL, Redis, etc.).

### Integration
A type-safe API client for third-party services, auto-generated from OpenAPI specifications with built-in authentication.

### Provider
An LLM service that supplies language models (OpenAI, Anthropic, Google, etc.). Providers are configured through the Vercel AI SDK.

## Message Terms

### Thread
A conversation session that groups related messages. Threads maintain context across multiple interactions.

### Message
A single communication unit in a conversation, containing role (user/assistant/system), content, and metadata.

### CoreMessage
The internal message format used by Mastra, compatible with various LLM provider formats.

### MessageList
An abstraction layer that handles message format conversion between different AI SDK versions and providers.

## Workflow Terms

### Run
An instance of workflow execution. Each run maintains its own state and can be suspended/resumed independently.

### Trigger
The event or data that initiates a workflow run. Can be manual, scheduled, or event-driven.

### Suspend
Pausing workflow execution to wait for external input or events. State is preserved for later resumption.

### Resume
Continuing a suspended workflow with new data. The workflow picks up from where it left off.

### Branch
A conditional path in a workflow where execution follows different steps based on conditions.

## Memory Terms

### Working Memory
Short-term context available during agent execution, typically the current conversation or task state.

### Semantic Recall
Retrieving relevant information based on meaning similarity rather than exact matches, using vector embeddings.

### Embedding
A numerical vector representation of text that captures semantic meaning for similarity comparisons.

### Vector Store
A specialized database for storing and querying high-dimensional vectors used in semantic search.

## RAG Terms

### RAG (Retrieval-Augmented Generation)
A technique that enhances LLM responses by retrieving relevant information from a knowledge base.

### Chunk
A segment of a document split for processing and embedding. Chunks typically overlap to preserve context.

### Reranking
Re-scoring retrieved documents using more sophisticated models to improve relevance.

### Knowledge Base
A collection of documents, embeddings, and metadata that agents can query for information.

## Tool Terms

### Schema
A Zod type definition that validates tool inputs and provides type safety.

### Executor
The function that implements a tool's logic, receiving validated parameters and returning results.

### Toolset
A collection of related tools, often provided by an integration.

### Tool Call
An invocation of a tool by an agent during generation, including parameters and results.

## Storage Terms

### Adapter
An implementation of the storage interface for a specific backend (PostgreSQL adapter, Redis adapter, etc.).

### Migration
A versioned schema change for storage backends, ensuring data structure consistency.

### Transaction
A group of storage operations that succeed or fail together, maintaining data consistency.

## Streaming Terms

### Stream
A sequence of data chunks delivered progressively rather than all at once.

### Chunk (streaming)
A small piece of data in a stream, such as a token or partial object update.

### Backpressure
Flow control mechanism preventing fast producers from overwhelming slow consumers.

### Server-Sent Events (SSE)
A protocol for server-to-client streaming over HTTP, used for real-time updates.

## Configuration Terms

### RuntimeContext
Request-scoped configuration and state that flows through execution, enabling dynamic behavior.

### DynamicArgument
A value that can be static or computed at runtime based on context.

### Mastra Instance
The central orchestrator that manages agents, workflows, storage, and other components.

## AI/LLM Terms

### Token
The basic unit of text that language models process, roughly corresponding to word parts.

### Context Window
The maximum number of tokens an LLM can process in a single request.

### Temperature
A parameter controlling randomness in LLM output (0 = deterministic, 1 = creative).

### System Prompt
Instructions that define an agent's behavior and personality, prepended to all interactions.

### Few-shot
Providing examples in the prompt to guide LLM behavior without training.

## Integration Terms

### OAuth
An authorization framework for granting third-party access without sharing credentials.

### Webhook
An HTTP callback that delivers real-time data when events occur in external systems.

### OpenAPI
A specification for describing REST APIs, used to generate type-safe integrations.

### Action
A method on an integration that corresponds to an API endpoint.

## Evaluation Terms

### Eval
An automated test that measures LLM output quality using various scoring methods.

### Metric
A quantitative measure of performance (accuracy, relevance, toxicity, etc.).

### Score
A normalized value (0-1) representing evaluation results.

### Model-graded
Evaluation where another LLM judges the output quality.

## Deployment Terms

### Deployer
An adapter that handles deployment to specific platforms (Vercel, Netlify, Cloudflare).

### Edge Runtime
JavaScript execution environment optimized for low latency at network edge locations.

### Serverless
Functions that run on-demand without managing servers, scaling automatically.

### Cold Start
Initial delay when a serverless function starts from an inactive state.

## Type Terms

### Zod
A TypeScript-first schema validation library used throughout Mastra.

### Type Guard
A function that narrows TypeScript types at runtime through validation.

### Generic
A type parameter that allows components to work with different types while maintaining type safety.

## Error Terms

### Retry
Automatically attempting an operation again after failure, with backoff strategies.

### Circuit Breaker
A pattern that stops calling failing services to prevent cascade failures.

### Idempotent
Operations that produce the same result regardless of how many times they're executed.

## Telemetry Terms

### Span
A unit of work in distributed tracing, representing a single operation.

### Trace
A collection of spans showing the complete path of a request through the system.

### OpenTelemetry
An observability framework for generating and collecting telemetry data.

## See Also

- [Mental Model](./mental-model.md)
- [Key Abstractions](./key-abstractions.md)
- [API Conventions](../02-api-reference/api-conventions.md)

## Next Steps

- [Explore the API](../02-api-reference/index.md)
- [Learn common patterns](../03-patterns/common-use-cases/index.md)
- [Understand integrations](../04-integration/frameworks/index.md)