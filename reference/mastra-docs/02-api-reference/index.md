# API Reference

> Complete API documentation for Mastra modules and components

## Overview

This section provides detailed API documentation for all Mastra modules. Each module has its own namespace with specific types, methods, and configuration options. All APIs are TypeScript-first with full type definitions.

## Core Modules

### [@mastra/core](./core/index.md)
The main framework package containing all essential components.

**Key Exports**:
- `Mastra` - Central orchestrator class
- `Agent` - AI agent implementation
- `Workflow` - Process orchestration
- `Step` - Workflow step definition
- `createTool` - Tool factory function

### [Agents](./agents/index.md)
AI agents with tool usage and memory integration.

**Key APIs**:
- `agent.generate()` - Generate text response
- `agent.stream()` - Stream text response
- `agent.generateObject()` - Generate structured data

### [Workflows](./workflows/index.md)
Durable state machines for complex processes.

**Key APIs**:
- `workflow.createRun()` - Create execution instance
- `step.execute()` - Step execution logic
- `run.suspend()` - Pause execution
- `run.resume()` - Continue execution

### [Tools](./tools/index.md)
Typed functions for extending agent capabilities.

**Key APIs**:
- `createTool()` - Define new tool
- `validateTool()` - Validate tool schema
- `tool.executor()` - Tool implementation

### [Memory](./memory/index.md)
Conversation persistence and context management.

**Key APIs**:
- `memory.saveMessages()` - Store messages
- `memory.getMessages()` - Retrieve history
- `memory.query()` - Semantic search

### [RAG](./rag/index.md)
Retrieval-augmented generation for knowledge bases.

**Key APIs**:
- `rag.add()` - Add documents
- `rag.query()` - Search knowledge
- `rag.delete()` - Remove documents

### [Integrations](./integrations/index.md)
Type-safe API clients for external services.

**Key APIs**:
- `integration.auth()` - Authenticate
- `integration.actions` - Available methods
- `integration.toolset` - Generated tools

## Storage Adapters

### PostgreSQL (`@mastra/pg`)
```typescript
import { PgStorage } from '@mastra/pg';
const storage = new PgStorage({ connectionString });
```

### Redis/Upstash (`@mastra/upstash`)
```typescript
import { UpstashStorage } from '@mastra/upstash';
const storage = new UpstashStorage({ url, token });
```

### DynamoDB (`@mastra/dynamodb`)
```typescript
import { DynamoDBStorage } from '@mastra/dynamodb';
const storage = new DynamoDBStorage({ region, table });
```

## Vector Stores

### Pinecone (`@mastra/pinecone`)
```typescript
import { PineconeVector } from '@mastra/pinecone';
const vector = new PineconeVector({ apiKey, index });
```

### ChromaDB (`@mastra/chroma`)
```typescript
import { ChromaVector } from '@mastra/chroma';
const vector = new ChromaVector({ url });
```

### Qdrant (`@mastra/qdrant`)
```typescript
import { QdrantVector } from '@mastra/qdrant';
const vector = new QdrantVector({ url, apiKey });
```

## Type Definitions

### Core Types
```typescript
type MastraConfig = {
  providers?: Record<string, Provider>;
  agents?: Record<string, AgentConfig>;
  workflows?: Record<string, WorkflowConfig>;
  storage?: Storage;
  memory?: Memory;
  tools?: Record<string, Tool>;
};

type AgentConfig = {
  model: string;
  instructions?: string;
  tools?: Record<string, Tool> | string[];
  memory?: MemoryConfig;
};

type Tool = {
  name: string;
  description: string;
  schema: ZodSchema;
  executor: (params: any) => Promise<any>;
};
```

## Method Signatures

### Agent Methods
```typescript
class Agent {
  generate(
    prompt: string | PromptOptions,
    options?: GenerateOptions
  ): Promise<GenerateResult>;

  stream(
    prompt: string | PromptOptions,
    options?: StreamOptions
  ): Promise<ReadableStream>;

  generateObject<T>(
    prompt: string,
    schema: ZodSchema<T>,
    options?: GenerateOptions
  ): Promise<T>;
}
```

### Workflow Methods
```typescript
class Workflow {
  createRun(): {
    start: (data?: any) => Promise<any>;
    suspend: () => Promise<SuspendData>;
    resume: (data?: any) => Promise<any>;
  };

  getStatus(runId: string): WorkflowStatus;
}
```

## Configuration Options

### Agent Options
```typescript
{
  model: 'openai:gpt-4o',         // Model identifier
  instructions: 'System prompt',   // Agent behavior
  temperature: 0.7,                // Creativity (0-1)
  maxTokens: 2000,                // Response limit
  tools: {},                       // Available tools
  memory: {                        // Memory config
    type: 'thread',
    threadId: 'thread-123'
  }
}
```

### Workflow Options
```typescript
{
  name: 'processOrder',           // Workflow name
  version: '1.0.0',               // Version
  steps: [],                      // Step definitions
  errorHandler: (error) => {},   // Error handling
  timeout: 30000,                 // Max duration
  retries: 3                      // Retry attempts
}
```

## Return Types

### GenerateResult
```typescript
type GenerateResult = {
  text: string;                   // Generated text
  usage: {                        // Token usage
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  finishReason: string;           // Stop reason
  toolCalls?: ToolCall[];         // Tool invocations
};
```

### StreamChunk
```typescript
type StreamChunk = {
  type: 'text' | 'tool' | 'error';
  content: string;
  metadata?: Record<string, any>;
};
```

## Error Types

```typescript
class MastraError extends Error {
  code: string;
  details?: any;
}

class AgentError extends MastraError {}
class WorkflowError extends MastraError {}
class StorageError extends MastraError {}
class IntegrationError extends MastraError {}
```

## Event Types

```typescript
// Workflow events
workflow.on('step:start', (data: StepStartEvent) => {});
workflow.on('step:complete', (data: StepCompleteEvent) => {});
workflow.on('step:error', (data: StepErrorEvent) => {});

// Agent events
agent.on('tool:start', (data: ToolStartEvent) => {});
agent.on('tool:complete', (data: ToolCompleteEvent) => {});
```

## See Also

- [API Conventions](./api-conventions.md)
- [Type Definitions](./types.md)
- [Error Handling](../03-patterns/error-handling.md)

## Next Steps

- [Explore agent APIs](./agents/index.md)
- [Learn workflow APIs](./workflows/index.md)
- [Understand tool APIs](./tools/index.md)