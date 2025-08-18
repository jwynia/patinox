# Minimal Example

> The smallest working Mastra application demonstrating core functionality

## Overview

This minimal example shows how to create an agent, define tools, and execute a workflow in under 30 lines of code. It demonstrates Mastra's core value proposition: rapid AI application development with type safety.

## Complete Working Example

```typescript
import { Mastra } from '@mastra/core';
import { createOpenAI } from '@ai-sdk/openai';
import { z } from 'zod';

// Initialize Mastra with OpenAI
const mastra = new Mastra({
  providers: {
    openai: createOpenAI({
      apiKey: process.env.OPENAI_API_KEY,
    }),
  },
  agents: {
    assistant: {
      model: 'openai:gpt-4o-mini',
      instructions: 'You are a helpful assistant.',
      tools: {
        getCurrentTime: {
          description: 'Get the current time',
          schema: z.object({}),
          executor: async () => ({ time: new Date().toISOString() }),
        },
      },
    },
  },
});

// Use the agent
async function main() {
  const agent = mastra.getAgent('assistant');
  const result = await agent.generate('What time is it?');
  console.log(result.text);
}

main().catch(console.error);
```

## Expected Output

```
The current time is 2024-03-15T10:30:45.123Z.
```

## Step-by-Step Breakdown

### 1. Import Dependencies
```typescript
import { Mastra } from '@mastra/core';
import { createOpenAI } from '@ai-sdk/openai';
import { z } from 'zod';
```

### 2. Configure Mastra Instance
```typescript
const mastra = new Mastra({
  providers: { /* LLM providers */ },
  agents: { /* Agent definitions */ },
});
```

### 3. Define Agent with Tools
```typescript
agents: {
  assistant: {
    model: 'openai:gpt-4o-mini',
    instructions: 'System prompt here',
    tools: { /* Tool definitions */ },
  },
}
```

### 4. Execute Agent
```typescript
const agent = mastra.getAgent('assistant');
const result = await agent.generate('Your prompt');
```

## Minimal Workflow Example

```typescript
import { Workflow, Step } from '@mastra/core';

const workflow = new Workflow({
  name: 'processData',
  steps: [
    new Step({
      id: 'fetch',
      execute: async () => ({ data: 'Hello' }),
    }),
    new Step({
      id: 'transform',
      execute: async ({ data }) => ({ result: data.toUpperCase() }),
    }),
  ],
});

// Run workflow
const { start } = workflow.createRun();
const result = await start();
console.log(result); // { result: 'HELLO' }
```

## Minimal RAG Example

```typescript
import { RAG } from '@mastra/rag';

const rag = new RAG({
  embedding: {
    provider: 'openai',
    model: 'text-embedding-3-small',
  },
  vectorStore: {
    provider: 'memory', // In-memory for testing
  },
});

// Add documents
await rag.add({ content: 'Mastra is an AI framework' });

// Query
const results = await rag.query('What is Mastra?');
```

## Running the Examples

1. Save code to `index.ts`
2. Set environment variables
3. Run with TypeScript:

```bash
npx tsx index.ts
```

## Common Patterns

### With Error Handling
```typescript
try {
  const result = await agent.generate('Query');
  console.log(result.text);
} catch (error) {
  console.error('Agent error:', error);
}
```

### With Streaming
```typescript
const stream = await agent.stream('Generate a story');
for await (const chunk of stream) {
  process.stdout.write(chunk);
}
```

### With Structured Output
```typescript
const schema = z.object({
  sentiment: z.enum(['positive', 'negative', 'neutral']),
  score: z.number(),
});

const result = await agent.generate('Analyze: Great product!', {
  output: schema,
});
console.log(result.object); // { sentiment: 'positive', score: 0.9 }
```

## See Also

- [Installation](./installation.md)
- [Import Patterns](./import-patterns.md)
- [Agent Configuration](../02-api-reference/agents/index.md)

## Next Steps

- [Add memory to agents](../02-api-reference/memory/index.md)
- [Create complex workflows](../03-patterns/composition-patterns.md)
- [Integrate external APIs](../04-integration/frameworks/index.md)