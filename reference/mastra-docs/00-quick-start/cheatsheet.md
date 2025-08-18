# Mastra Cheatsheet

> Quick reference for common Mastra operations and patterns

## Core Operations

### Initialize Mastra
```typescript
import { Mastra } from '@mastra/core';
const mastra = new Mastra({ providers, agents, workflows });
```

### Create Agent
```typescript
const agent = mastra.getAgent('assistant');
await agent.generate('prompt');
await agent.stream('prompt');
```

### Define Tool
```typescript
tools: {
  myTool: {
    description: 'Tool description',
    schema: z.object({ param: z.string() }),
    executor: async ({ param }) => ({ result: param }),
  }
}
```

### Create Workflow
```typescript
const workflow = new Workflow({
  name: 'myWorkflow',
  steps: [new Step({ id: 'step1', execute: async () => ({}) })]
});
```

## Agent Methods

```typescript
// Generate text
const result = await agent.generate('prompt');
result.text;           // Generated text
result.usage;          // Token usage
result.toolCalls;      // Tool invocations

// Stream response
const stream = await agent.stream('prompt');
for await (const chunk of stream) { }

// Structured output
await agent.generate('prompt', {
  output: z.object({ field: z.string() })
});

// With messages
await agent.generate({
  messages: [
    { role: 'user', content: 'Hello' },
    { role: 'assistant', content: 'Hi' }
  ]
});

// With system prompt
await agent.generate({
  system: 'You are helpful',
  prompt: 'User query'
});
```

## Workflow Operations

```typescript
// Create and run
const { start } = workflow.createRun();
const result = await start({ triggerData: {} });

// Suspend and resume
const { suspend, resume } = workflow.createRun();
const suspendData = await suspend();
await resume({ resumeData: {} });

// Get status
const status = workflow.getStatus(runId);
```

## Memory Operations

```typescript
// Initialize memory
import { MastraMemory } from '@mastra/memory';
const memory = new MastraMemory({ storage });

// Save to thread
await memory.saveMessages({
  threadId: 'thread-1',
  messages: [{ role: 'user', content: 'Hello' }]
});

// Retrieve messages
const messages = await memory.getMessages({ threadId: 'thread-1' });

// Semantic search
const results = await memory.query({
  query: 'search text',
  limit: 10
});
```

## Tool Patterns

```typescript
// Simple tool
{
  description: 'Get time',
  schema: z.object({}),
  executor: async () => ({ time: Date.now() })
}

// Tool with parameters
{
  description: 'Add numbers',
  schema: z.object({ a: z.number(), b: z.number() }),
  executor: async ({ a, b }) => ({ sum: a + b })
}

// Async tool with API call
{
  description: 'Fetch data',
  schema: z.object({ url: z.string() }),
  executor: async ({ url }) => {
    const res = await fetch(url);
    return res.json();
  }
}
```

## Storage Setup

```typescript
// PostgreSQL
import { PgStorage } from '@mastra/pg';
const storage = new PgStorage({
  connectionString: process.env.DATABASE_URL
});

// Upstash
import { UpstashStorage } from '@mastra/upstash';
const storage = new UpstashStorage({
  url: process.env.UPSTASH_URL,
  token: process.env.UPSTASH_TOKEN
});
```

## Vector Store Setup

```typescript
// Pinecone
import { PineconeVector } from '@mastra/pinecone';
const vector = new PineconeVector({
  apiKey: process.env.PINECONE_API_KEY,
  index: 'my-index'
});

// ChromaDB
import { ChromaVector } from '@mastra/chroma';
const vector = new ChromaVector({
  url: 'http://localhost:8000'
});
```

## RAG Operations

```typescript
import { RAG } from '@mastra/rag';

const rag = new RAG({
  embedding: { provider: 'openai', model: 'text-embedding-3-small' },
  vectorStore: { provider: 'pinecone' }
});

// Add documents
await rag.add({ content: 'Document text' });

// Query
const results = await rag.query('Search query');
```

## Error Handling

```typescript
// Try-catch
try {
  await agent.generate('prompt');
} catch (error) {
  console.error('Error:', error.message);
}

// With retries
await agent.generate('prompt', {
  maxRetries: 3,
  abortSignal: AbortSignal.timeout(5000)
});
```

## Streaming Patterns

```typescript
// Text stream
const stream = await agent.stream('prompt');
for await (const chunk of stream) {
  process.stdout.write(chunk);
}

// Object stream
const stream = await agent.streamObject('prompt', {
  schema: z.object({ data: z.string() })
});
for await (const obj of stream) {
  console.log(obj);
}
```

## Provider Configuration

```typescript
// Multiple providers
{
  providers: {
    openai: createOpenAI({ apiKey: 'sk-...' }),
    anthropic: createAnthropic({ apiKey: 'sk-ant-...' }),
    google: createGoogleGenerativeAI({ apiKey: '...' })
  }
}

// Model selection
agent: {
  model: 'openai:gpt-4o',           // OpenAI
  model: 'anthropic:claude-3-opus',  // Anthropic
  model: 'google:gemini-pro',        // Google
}
```

## Environment Variables

```bash
# .env file
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_GENERATIVE_AI_API_KEY=...
DATABASE_URL=postgresql://...
REDIS_URL=redis://...
PINECONE_API_KEY=...
```

## Common Commands

```bash
# Create new project
npx create-mastra@latest

# Start dev server
npm run dev
mastra dev

# Build project
npm run build

# Run tests
npm test

# Type check
npm run typecheck
```

## Type Imports

```typescript
import type {
  MastraConfig,
  AgentConfig,
  WorkflowConfig,
  Tool,
  Step,
  CoreMessage,
  StreamTextResult,
  GenerateTextResult
} from '@mastra/core';
```