# Common Mistakes

> Frequent errors when using Mastra and their solutions

## Overview

This guide covers the most common mistakes developers make when using Mastra and how to avoid them. Learning from these patterns will save debugging time and improve application reliability.

## Agent Mistakes

### 1. Not Awaiting Async Operations

❌ **Wrong**:
```typescript
// Missing await - returns Promise instead of result
const result = agent.generate('prompt'); 
console.log(result.text); // undefined!
```

✅ **Correct**:
```typescript
const result = await agent.generate('prompt');
console.log(result.text); // Works
```

### 2. Reusing Agent Instances Incorrectly

❌ **Wrong**:
```typescript
// Creating new agent for every request
async function handleRequest(prompt: string) {
  const agent = new Agent({ model: 'gpt-4' }); // Expensive!
  return agent.generate(prompt);
}
```

✅ **Correct**:
```typescript
// Reuse agent instance
const agent = mastra.getAgent('assistant');

async function handleRequest(prompt: string) {
  return agent.generate(prompt);
}
```

### 3. Ignoring Token Limits

❌ **Wrong**:
```typescript
// Sending huge context without checking
const result = await agent.generate({
  messages: last1000Messages // May exceed context window!
});
```

✅ **Correct**:
```typescript
// Manage context window
const recentMessages = messages.slice(-10); // Last 10 messages
const result = await agent.generate({
  messages: recentMessages,
  maxTokens: 1000
});
```

### 4. Not Handling Tool Errors

❌ **Wrong**:
```typescript
const tool = {
  executor: async (params) => {
    return fetchData(params.url); // May throw!
  }
};
```

✅ **Correct**:
```typescript
const tool = {
  executor: async (params) => {
    try {
      return await fetchData(params.url);
    } catch (error) {
      return { error: error.message };
    }
  }
};
```

## Workflow Mistakes

### 1. Not Handling Suspend/Resume Properly

❌ **Wrong**:
```typescript
// Losing suspend data
const { start } = workflow.createRun();
await start(); // Ignoring potential suspension
```

✅ **Correct**:
```typescript
const { start, resume } = workflow.createRun();
const result = await start();

if (result.status === 'suspended') {
  // Store suspend data
  await storage.set('suspend-data', result.suspendData);
  
  // Later, resume with data
  const resumeData = await getUserInput();
  await resume(resumeData);
}
```

### 2. Creating Steps with Side Effects

❌ **Wrong**:
```typescript
// Step modifies external state directly
new Step({
  execute: async (data) => {
    globalCounter++; // Side effect!
    database.write(data); // Not idempotent!
    return data;
  }
});
```

✅ **Correct**:
```typescript
// Pure, idempotent step
new Step({
  execute: async (data) => {
    // Return new state, don't mutate
    const result = processData(data);
    
    // Use workflow context for state
    await context.storage.set('counter', count + 1);
    
    return result;
  }
});
```

### 3. Infinite Loops in Workflows

❌ **Wrong**:
```typescript
// No exit condition
const workflow = new Workflow({
  steps: [
    new Step({
      id: 'loop',
      execute: async (data) => {
        return { next: 'loop' }; // Infinite!
      }
    })
  ]
});
```

✅ **Correct**:
```typescript
// With exit condition
const workflow = new Workflow({
  steps: [
    new Step({
      id: 'loop',
      execute: async (data) => {
        if (data.count >= 10) {
          return { next: 'done' };
        }
        return { next: 'loop', count: data.count + 1 };
      }
    })
  ]
});
```

## Memory Mistakes

### 1. Not Specifying Thread ID

❌ **Wrong**:
```typescript
// No thread ID - creates new thread each time
await agent.generate('Hello');
await agent.generate('Follow up'); // No context!
```

✅ **Correct**:
```typescript
// Use consistent thread ID
const threadId = 'conversation-123';

await agent.generate('Hello', { threadId });
await agent.generate('Follow up', { threadId }); // Has context
```

### 2. Storing Large Objects in Memory

❌ **Wrong**:
```typescript
// Storing entire files in messages
await memory.saveMessages([{
  role: 'user',
  content: JSON.stringify(largeFile) // Too big!
}]);
```

✅ **Correct**:
```typescript
// Store reference, not content
await memory.saveMessages([{
  role: 'user',
  content: 'Processing file',
  metadata: { fileId: 'file-123', size: file.size }
}]);
```

### 3. Not Cleaning Up Old Threads

❌ **Wrong**:
```typescript
// Creating threads without cleanup
function createChat() {
  return `thread-${Date.now()}`; // Accumulates forever!
}
```

✅ **Correct**:
```typescript
// Implement cleanup strategy
async function cleanupOldThreads() {
  const threads = await storage.listThreads();
  const oneWeekAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;
  
  for (const thread of threads) {
    if (thread.updatedAt < oneWeekAgo) {
      await storage.deleteThread(thread.id);
    }
  }
}
```

## Storage Mistakes

### 1. Not Handling Connection Failures

❌ **Wrong**:
```typescript
// Assuming storage always works
const data = await storage.get('key'); // May fail!
processData(data);
```

✅ **Correct**:
```typescript
// Handle storage failures
try {
  const data = await storage.get('key');
  if (data) {
    processData(data);
  } else {
    // Handle missing data
    useDefaultData();
  }
} catch (error) {
  // Handle connection error
  console.error('Storage error:', error);
  useFallback();
}
```

### 2. Using Wrong Storage for Use Case

❌ **Wrong**:
```typescript
// Using key-value store for complex queries
const users = await storage.get('users');
const filtered = users.filter(u => u.age > 18); // Inefficient!
```

✅ **Correct**:
```typescript
// Use appropriate storage type
const users = await database.query({
  table: 'users',
  where: { age: { gt: 18 } }
});
```

## Tool Mistakes

### 1. Missing Schema Validation

❌ **Wrong**:
```typescript
const tool = {
  executor: async (params) => {
    // No validation - params could be anything!
    return fetchUser(params.id);
  }
};
```

✅ **Correct**:
```typescript
const tool = createTool({
  schema: z.object({
    id: z.string().uuid()
  }),
  executor: async (params) => {
    // params.id is guaranteed to be valid UUID
    return fetchUser(params.id);
  }
});
```

### 2. Blocking Operations in Tools

❌ **Wrong**:
```typescript
const tool = {
  executor: (params) => {
    // Synchronous blocking operation
    const result = fs.readFileSync(params.path);
    return result;
  }
};
```

✅ **Correct**:
```typescript
const tool = {
  executor: async (params) => {
    // Async non-blocking
    const result = await fs.promises.readFile(params.path);
    return result;
  }
};
```

## Configuration Mistakes

### 1. Hardcoding API Keys

❌ **Wrong**:
```typescript
const mastra = new Mastra({
  providers: {
    openai: createOpenAI({
      apiKey: 'sk-1234567890' // Never do this!
    })
  }
});
```

✅ **Correct**:
```typescript
// Use environment variables
const mastra = new Mastra({
  providers: {
    openai: createOpenAI({
      apiKey: process.env.OPENAI_API_KEY
    })
  }
});
```

### 2. Not Setting Appropriate Timeouts

❌ **Wrong**:
```typescript
// No timeout - could hang forever
await agent.generate(prompt);
```

✅ **Correct**:
```typescript
// Set reasonable timeout
await agent.generate(prompt, {
  abortSignal: AbortSignal.timeout(30000) // 30 seconds
});
```

## Streaming Mistakes

### 1. Not Consuming Streams

❌ **Wrong**:
```typescript
// Creating stream but not consuming it
const stream = await agent.stream(prompt);
// Stream is created but not read - resources leak!
```

✅ **Correct**:
```typescript
// Always consume or close streams
const stream = await agent.stream(prompt);
try {
  for await (const chunk of stream) {
    process.stdout.write(chunk);
  }
} finally {
  stream.close?.(); // Ensure cleanup
}
```

### 2. Buffering Entire Stream

❌ **Wrong**:
```typescript
// Loading entire stream into memory
const chunks = [];
for await (const chunk of stream) {
  chunks.push(chunk); // Memory grows!
}
const result = chunks.join('');
```

✅ **Correct**:
```typescript
// Process stream incrementally
for await (const chunk of stream) {
  processChunk(chunk); // Process and discard
}
```

## Type Mistakes

### 1. Using 'any' Type

❌ **Wrong**:
```typescript
function processData(data: any) {
  return data.value; // No type safety!
}
```

✅ **Correct**:
```typescript
interface Data {
  value: string;
}

function processData(data: Data) {
  return data.value; // Type safe
}
```

### 2. Ignoring TypeScript Errors

❌ **Wrong**:
```typescript
// @ts-ignore
const result = await agent.gnerate(prompt); // Typo hidden!
```

✅ **Correct**:
```typescript
// Fix the actual issue
const result = await agent.generate(prompt);
```

## Performance Mistakes

### 1. Not Caching Expensive Operations

❌ **Wrong**:
```typescript
// Regenerating embeddings every time
async function search(query: string) {
  const embedding = await generateEmbedding(query);
  return vectorStore.search(embedding);
}
```

✅ **Correct**:
```typescript
// Cache embeddings
const embeddingCache = new Map();

async function search(query: string) {
  let embedding = embeddingCache.get(query);
  if (!embedding) {
    embedding = await generateEmbedding(query);
    embeddingCache.set(query, embedding);
  }
  return vectorStore.search(embedding);
}
```

## See Also

- [Performance Pitfalls](./performance-pitfalls.md)
- [Security Considerations](./security-considerations.md)
- [Troubleshooting Guide](./troubleshooting.md)

## Next Steps

- [Review performance patterns](../03-patterns/performance-patterns.md)
- [Implement error handling](../03-patterns/error-handling.md)
- [Set up testing](../04-integration/testing-setup.md)