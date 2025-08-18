# Architecture Overview

> How Mastra is organized and its main architectural components

## Overview

Mastra follows a modular, plugin-based architecture with a central orchestration hub. Components are loosely coupled through dependency injection, allowing flexible composition of AI applications. The framework emphasizes type safety, streaming capabilities, and serverless deployment.

## Core Architecture Principles

### 1. Central Orchestration
The `Mastra` class acts as the central configuration and dependency injection container. All components register with and are accessed through this hub.

```typescript
const mastra = new Mastra({
  providers,   // LLM providers
  agents,      // Agent definitions
  workflows,   // Workflow configurations
  storage,     // Persistence layer
  memory,      // Memory systems
  tools,       // Available tools
});
```

### 2. Plugin Architecture
Components implement standardized interfaces, making them interchangeable:
- Storage adapters (PostgreSQL, Redis, etc.)
- Vector stores (Pinecone, Chroma, etc.)
- LLM providers (OpenAI, Anthropic, etc.)
- Memory systems (thread-based, semantic)

### 3. Runtime Context
Request-scoped context propagation enables dynamic configuration:
```typescript
const runtimeContext = new RuntimeContext({
  userId: 'user-123',
  sessionId: 'session-456',
  environment: 'production',
});
```

## Component Hierarchy

```
Mastra (Orchestrator)
├── Agents (AI Interaction)
│   ├── LLM Provider
│   ├── Tools
│   ├── Memory
│   └── Message Processing
├── Workflows (Process Orchestration)
│   ├── Steps
│   ├── State Management
│   └── Suspend/Resume
├── Storage (Persistence)
│   ├── Key-Value Store
│   ├── Document Store
│   └── Vector Store
├── Memory (Conversation State)
│   ├── Thread Management
│   ├── Message History
│   └── Semantic Recall
├── RAG (Knowledge Base)
│   ├── Document Processing
│   ├── Embedding
│   └── Vector Search
└── Integrations (External APIs)
    ├── OAuth Providers
    ├── API Clients
    └── Webhooks
```

## Data Flow Architecture

### Agent Request Flow
```
User Input → Agent → LLM Provider → Tool Execution → Memory Save → Response
                ↑                          ↓
                └─────── Tool Results ←────┘
```

### Workflow Execution Flow
```
Trigger → Step 1 → Step 2 → Decision → [Branch A/B] → Final Step
             ↓         ↓                    ↓
          Storage  Storage              Suspend Point
```

## Component Interactions

### Agent-Memory Interaction
Agents automatically persist conversations to memory:
1. User message received
2. Message saved to thread
3. Context retrieved from memory
4. LLM generates response
5. Response saved to thread

### Workflow-Agent Collaboration
Workflows can invoke agents as steps:
```typescript
new Step({
  id: 'analyze',
  execute: async (data) => {
    const agent = mastra.getAgent('analyst');
    return agent.generate(`Analyze: ${data.text}`);
  }
})
```

## Scalability Architecture

### Stateless Design
- Agents and workflows are stateless
- State persisted to external storage
- Horizontal scaling supported

### Streaming Architecture
- Server-sent events for real-time updates
- Chunked responses for large outputs
- Backpressure handling

### Serverless Deployment
- Function-based execution model
- Cold start optimization
- Edge runtime compatibility

## Extension Points

### Custom Storage Adapters
```typescript
class CustomStorage extends BaseStorage {
  async get(key: string) { /* implementation */ }
  async set(key: string, value: any) { /* implementation */ }
}
```

### Custom Tools
```typescript
const customTool = createTool({
  name: 'myTool',
  description: 'Custom functionality',
  schema: z.object({ /* params */ }),
  executor: async (params) => { /* logic */ }
});
```

### Custom Memory Systems
```typescript
class CustomMemory extends MastraMemory {
  async recall(query: string) { /* implementation */ }
}
```

## Message List Abstraction

Unified message handling across different formats:
- Vercel AI SDK v4/v5 messages
- OpenAI format
- Anthropic format
- Internal Mastra format

## Telemetry Architecture

### OpenTelemetry Integration
- Automatic span creation
- Distributed tracing
- Metrics collection
- Custom attributes

### Observability Points
- Agent execution traces
- Tool invocation metrics
- Workflow step timing
- Memory operation logging

## Security Architecture

### API Key Management
- Environment variable isolation
- Runtime provider configuration
- Secure token storage

### Input Validation
- Schema validation with Zod
- Parameter sanitization
- Rate limiting support

## Performance Optimizations

### Caching Strategies
- LLM response caching
- Embedding cache
- Tool result memoization

### Batch Processing
- Bulk message operations
- Parallel tool execution
- Concurrent step processing

## See Also

- [Key Abstractions](./key-abstractions.md)
- [Data Flow](./data-flow.md)
- [Mental Model](./mental-model.md)

## Next Steps

- [Understand core abstractions](./key-abstractions.md)
- [Learn the mental model](./mental-model.md)
- [Explore API structure](../02-api-reference/index.md)