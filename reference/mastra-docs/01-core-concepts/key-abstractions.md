# Key Abstractions

> Main entities in Mastra and their relationships

## Overview

Mastra provides six core abstractions that work together to build AI applications: Agents, Tools, Workflows, Memory, Storage, and Integrations. Each abstraction has a specific purpose and well-defined interfaces.

## Agent

**Purpose**: Primary interface for LLM interaction with tool usage and memory.

```typescript
interface Agent {
  generate(prompt: string, options?: GenerateOptions): Promise<GenerateResult>;
  stream(prompt: string, options?: StreamOptions): Promise<ReadableStream>;
  generateObject<T>(prompt: string, schema: ZodSchema<T>): Promise<T>;
}
```

**Key Characteristics**:
- Stateless execution model
- Automatic tool orchestration
- Memory persistence
- Streaming support
- Structured output generation

**Relationships**:
- Uses **Tools** for extended capabilities
- Persists to **Memory** for context
- Configured through **Mastra** instance

## Tool

**Purpose**: Typed functions that extend agent capabilities.

```typescript
interface Tool {
  name: string;
  description: string;
  schema: ZodSchema;
  executor: (params: any) => Promise<any>;
}
```

**Key Characteristics**:
- Schema-validated inputs
- Async execution
- Integration access
- Composable with agents and workflows

**Relationships**:
- Executed by **Agents** during generation
- Used in **Workflow** steps
- Access **Integrations** for external APIs

## Workflow

**Purpose**: Durable, graph-based state machines for complex processes.

```typescript
interface Workflow {
  name: string;
  steps: Step[];
  createRun(): WorkflowRun;
}
```

**Key Characteristics**:
- Step-based execution
- Branching and loops
- Suspend/resume capability
- Error handling and retries
- Built-in telemetry

**Relationships**:
- Orchestrates **Steps**
- Can invoke **Agents**
- Persists state to **Storage**

## Step

**Purpose**: Individual units of work within workflows.

```typescript
interface Step<TInput, TOutput> {
  id: string;
  execute: (input: TInput) => Promise<TOutput>;
  schema?: ZodSchema<TInput>;
}
```

**Key Characteristics**:
- Typed input/output
- Async execution
- Error boundaries
- Retry logic

**Relationships**:
- Composed into **Workflows**
- Can use **Tools**
- Access **RuntimeContext**

## Memory

**Purpose**: Thread-based conversation persistence with semantic recall.

```typescript
interface Memory {
  saveMessages(messages: Message[]): Promise<void>;
  getMessages(threadId: string): Promise<Message[]>;
  query(text: string, options?: QueryOptions): Promise<Result[]>;
}
```

**Key Characteristics**:
- Thread management
- Message history
- Semantic search
- Working memory
- Cross-session persistence

**Relationships**:
- Used by **Agents** for context
- Backed by **Storage**
- Utilizes **Vector** stores

## Storage

**Purpose**: Pluggable persistence layer with standardized interfaces.

```typescript
interface Storage {
  get<T>(key: string): Promise<T | null>;
  set<T>(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  list(prefix?: string): Promise<string[]>;
}
```

**Key Characteristics**:
- Key-value operations
- Document storage
- Query capabilities
- Transaction support
- Multiple backend adapters

**Relationships**:
- Backs **Memory** system
- Stores **Workflow** state
- Persists **Agent** conversations

## Integration

**Purpose**: Type-safe API clients for third-party services.

```typescript
interface Integration {
  name: string;
  auth: AuthConfig;
  actions: Record<string, Action>;
  toolset: Tool[];
}
```

**Key Characteristics**:
- Auto-generated from OpenAPI
- OAuth/API key authentication
- Type-safe methods
- Tool generation

**Relationships**:
- Provides **Tools** to agents
- Used in **Workflow** steps
- Managed by **Mastra** instance

## RAG (Retrieval-Augmented Generation)

**Purpose**: Knowledge base construction and querying.

```typescript
interface RAG {
  add(document: Document): Promise<void>;
  query(text: string, limit?: number): Promise<Document[]>;
  delete(id: string): Promise<void>;
}
```

**Key Characteristics**:
- Document chunking
- Embedding generation
- Vector search
- Reranking strategies

**Relationships**:
- Enhances **Agent** knowledge
- Uses **Vector** stores
- Integrates with **Memory**

## Vector Store

**Purpose**: Semantic search and similarity matching.

```typescript
interface VectorStore {
  upsert(vectors: Vector[]): Promise<void>;
  query(vector: number[], limit: number): Promise<Match[]>;
  delete(ids: string[]): Promise<void>;
}
```

**Key Characteristics**:
- High-dimensional vectors
- Similarity metrics
- Filtering capabilities
- Multiple provider support

**Relationships**:
- Powers **RAG** search
- Enables **Memory** semantic recall
- Backed by specialized databases

## RuntimeContext

**Purpose**: Request-scoped configuration and state.

```typescript
interface RuntimeContext {
  userId?: string;
  sessionId?: string;
  environment?: string;
  metadata?: Record<string, any>;
}
```

**Key Characteristics**:
- Thread-safe isolation
- Dynamic configuration
- Cross-component propagation

**Relationships**:
- Passed to **Tools** and **Steps**
- Available in **Agents**
- Flows through **Workflows**

## Entity Relationships

```
Mastra
  ├── Agent[] ←──uses──→ Tool[]
  │     ↓                   ↑
  │   saves              provided
  │     ↓                   ↑
  ├── Memory ←──backed──→ Storage
  │     ↓
  │   queries
  │     ↓
  ├── Vector Store
  │
  ├── Workflow[] ←──contains──→ Step[]
  │                                ↓
  │                              uses
  │                                ↓
  └── Integration[] ──provides──→ Tool[]
```

## Composition Examples

### Agent with Tools and Memory
```typescript
const agent = new Agent({
  model: 'gpt-4',
  tools: [weatherTool, calculatorTool],
  memory: threadMemory,
});
```

### Workflow with Agent Steps
```typescript
const workflow = new Workflow({
  steps: [
    new Step({
      execute: async (data) => {
        const agent = mastra.getAgent('assistant');
        return agent.generate(data.prompt);
      }
    })
  ]
});
```

### RAG-Enhanced Agent
```typescript
const agent = new Agent({
  tools: [
    {
      name: 'search_knowledge',
      executor: async ({ query }) => {
        return rag.query(query, { limit: 5 });
      }
    }
  ]
});
```

## See Also

- [Architecture Overview](./architecture-overview.md)
- [Data Flow](./data-flow.md)
- [Mental Model](./mental-model.md)

## Next Steps

- [Explore agent capabilities](../02-api-reference/agents/index.md)
- [Build workflows](../02-api-reference/workflows/index.md)
- [Implement memory systems](../02-api-reference/memory/index.md)