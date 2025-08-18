# Memory API Reference

> Complete API documentation for Memory systems and conversation management

## Overview

Mastra Memory provides thread-based conversation persistence with semantic recall capabilities. It manages message history, working memory, and cross-session context. Memory supports multiple storage backends and includes vector-based semantic search for relevant information retrieval.

## Class: MastraMemory

### Constructor

```typescript
new MastraMemory(config: MemoryConfig)
```

**Parameters**:
- `config: MemoryConfig` - Memory configuration object

**Example**:
```typescript
const memory = new MastraMemory({
  name: 'conversation-memory',
  storage: pgStorage,
  vectorStore: pineconeVector,
  embedding: {
    provider: 'openai',
    model: 'text-embedding-3-small'
  },
  maxMessages: 1000,
  maxTokens: 8000
});
```

## Core Methods

### saveMessages

Store messages in memory with automatic thread management.

```typescript
// Overloaded signatures
saveMessages(args: SaveMessagesArgs & { format?: 'v1' }): Promise<MastraMessageV1[]>
saveMessages(args: SaveMessagesArgs & { format: 'v2' }): Promise<MastraMessageV2[]>
```

**Parameters**:
- `messages: Message[]` - Array of messages to store
- `threadId: string` - Thread identifier
- `resourceId?: string` - Optional resource context
- `format?: 'v1' | 'v2'` - Message format version

**Returns**: `Promise<Message[]>` - Stored messages with IDs

**Example**:
```typescript
// Basic message saving
await memory.saveMessages({
  threadId: 'conversation-123',
  messages: [
    { role: 'user', content: 'Hello, how are you?' },
    { role: 'assistant', content: 'I am doing well, thank you!' }
  ]
});

// With resource context
await memory.saveMessages({
  threadId: 'support-456',
  resourceId: 'ticket-789',
  messages: [
    { 
      role: 'user', 
      content: 'I need help with my order',
      metadata: { orderId: 'order-123' }
    }
  ]
});
```

### getMessages

Retrieve message history from memory.

```typescript
// Overloaded signatures  
getMessages(args: GetMessagesArgs & { format?: 'v1' }): Promise<MastraMessageV1[]>
getMessages(args: GetMessagesArgs & { format: 'v2' }): Promise<MastraMessageV2[]>
```

**Parameters**:
- `threadId: string` - Thread identifier
- `resourceId?: string` - Optional resource filter
- `limit?: number` - Maximum messages to return
- `before?: string` - Get messages before this ID
- `after?: string` - Get messages after this ID
- `format?: 'v1' | 'v2'` - Message format version

**Returns**: `Promise<Message[]>` - Retrieved messages

**Example**:
```typescript
// Get recent messages
const messages = await memory.getMessages({
  threadId: 'conversation-123',
  limit: 20
});

// Get messages with pagination
const olderMessages = await memory.getMessages({
  threadId: 'conversation-123',
  before: lastMessageId,
  limit: 10
});

// Get messages for specific resource
const ticketMessages = await memory.getMessages({
  threadId: 'support-456',
  resourceId: 'ticket-789'
});
```

### query

Perform semantic search across memory.

```typescript
query(options: QueryOptions): Promise<QueryResult[]>
```

**Parameters**:
- `query: string` - Search text
- `threadId?: string` - Limit to specific thread
- `resourceId?: string` - Limit to specific resource
- `limit?: number` - Maximum results (default: 10)
- `threshold?: number` - Similarity threshold (0-1)
- `timeRange?: { from: Date, to: Date }` - Time filtering

**Returns**: `Promise<QueryResult[]>` - Relevant messages with scores

**Example**:
```typescript
// Semantic search across all memory
const results = await memory.query({
  query: 'user mentioned payment issues',
  limit: 5,
  threshold: 0.7
});

// Search within specific thread
const threadResults = await memory.query({
  query: 'shipping address',
  threadId: 'conversation-123',
  limit: 3
});

// Time-bounded search
const recentIssues = await memory.query({
  query: 'technical problems',
  timeRange: {
    from: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000), // Last week
    to: new Date()
  }
});
```

### recall

Get contextually relevant messages for a conversation.

```typescript
recall(options: RecallOptions): Promise<RecallResult>
```

**Parameters**:
- `threadId: string` - Thread to search within
- `context: string` - Current conversation context
- `limit?: number` - Maximum messages to recall
- `relevanceThreshold?: number` - Similarity threshold

**Returns**: `Promise<RecallResult>` - Relevant historical context

**Example**:
```typescript
const context = await memory.recall({
  threadId: 'conversation-123',
  context: 'user asking about their previous order',
  limit: 5,
  relevanceThreshold: 0.6
});

console.log('Relevant history:', context.messages);
console.log('Context summary:', context.summary);
```

## Thread Management

### createThread

Create a new conversation thread.

```typescript
createThread(options: CreateThreadOptions): Promise<Thread>
```

**Example**:
```typescript
const thread = await memory.createThread({
  name: 'Customer Support - John Doe',
  metadata: {
    customerId: 'customer-123',
    channel: 'email',
    priority: 'high'
  }
});
```

### getThread

Retrieve thread information.

```typescript
getThread(threadId: string): Promise<Thread | null>
```

### updateThread

Update thread metadata.

```typescript
updateThread(threadId: string, updates: Partial<Thread>): Promise<Thread>
```

### deleteThread

Remove thread and all associated messages.

```typescript
deleteThread(threadId: string): Promise<void>
```

## Working Memory

### setWorkingMemory

Store temporary context for current session.

```typescript
setWorkingMemory(key: string, value: any, ttl?: number): Promise<void>
```

**Example**:
```typescript
// Store user preferences for session
await memory.setWorkingMemory('user-preferences', {
  language: 'en',
  timezone: 'UTC-8',
  theme: 'dark'
}, 3600000); // 1 hour TTL

// Store conversation state
await memory.setWorkingMemory(`state:${threadId}`, {
  currentTopic: 'order-tracking',
  lastAction: 'lookup-order',
  orderNumber: 'ORD-123'
});
```

### getWorkingMemory

Retrieve temporary context.

```typescript
getWorkingMemory<T>(key: string): Promise<T | null>
```

### clearWorkingMemory

Remove temporary context.

```typescript
clearWorkingMemory(key: string): Promise<void>
```

## Configuration Types

### MemoryConfig

```typescript
interface MemoryConfig {
  name: string;
  storage: Storage;
  vectorStore?: VectorStore;
  embedding?: {
    provider: string;
    model: string;
    apiKey?: string;
  };
  maxMessages?: number;
  maxTokens?: number;
  retentionDays?: number;
  compressionThreshold?: number;
  enableSemanticSearch?: boolean;
}
```

### Message Types

```typescript
// V1 Message Format
interface MastraMessageV1 {
  id: string;
  threadId: string;
  resourceId?: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  metadata?: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

// V2 Message Format (Enhanced)
interface MastraMessageV2 extends MastraMessageV1 {
  parentId?: string;
  toolCalls?: ToolCall[];
  citations?: Citation[];
  sentiment?: {
    score: number;
    label: 'positive' | 'negative' | 'neutral';
  };
  embedding?: number[];
}
```

### Query Types

```typescript
interface QueryOptions {
  query: string;
  threadId?: string;
  resourceId?: string;
  limit?: number;
  threshold?: number;
  timeRange?: {
    from: Date;
    to: Date;
  };
  includeMetadata?: boolean;
}

interface QueryResult {
  message: MastraMessage;
  score: number;
  explanation?: string;
  context?: {
    before: MastraMessage[];
    after: MastraMessage[];
  };
}
```

## Advanced Features

### Message Compression

Automatically compress old messages to save storage:

```typescript
const memory = new MastraMemory({
  compressionThreshold: 100, // Compress after 100 messages
  retentionDays: 30,
  // Compression strategy
  compression: {
    enabled: true,
    algorithm: 'gzip',
    summaryModel: 'gpt-3.5-turbo'
  }
});
```

### Context Windows

Manage token limits automatically:

```typescript
const contextMessages = await memory.getContextWindow({
  threadId: 'conversation-123',
  maxTokens: 4000,
  strategy: 'recent-with-relevance' // 'recent' | 'relevant' | 'recent-with-relevance'
});
```

### Message Filtering

```typescript
// Filter by role
const userMessages = await memory.getMessages({
  threadId: 'conversation-123',
  filter: { role: 'user' }
});

// Filter by metadata
const importantMessages = await memory.getMessages({
  threadId: 'conversation-123',
  filter: { 
    metadata: { priority: 'high' }
  }
});

// Custom filter function
const filteredMessages = await memory.getMessages({
  threadId: 'conversation-123',
  filter: (message) => {
    return message.content.includes('urgent') || 
           message.metadata?.priority === 'critical';
  }
});
```

## Integration Patterns

### With Agents

```typescript
// Agent with automatic memory integration
const agent = new Agent({
  model: 'gpt-4',
  memory: {
    type: 'thread',
    instance: memory
  }
});

// Messages automatically saved/retrieved
await agent.generate('Hello', { threadId: 'conversation-123' });
```

### With RAG

```typescript
// Combine memory with knowledge base
const enhancedRecall = async (threadId: string, query: string) => {
  // Get conversation context
  const memoryResults = await memory.query({
    query,
    threadId,
    limit: 3
  });
  
  // Get knowledge base results
  const ragResults = await rag.query(query, { limit: 5 });
  
  // Combine and rank
  return rankAndCombine(memoryResults, ragResults);
};
```

### With Workflows

```typescript
const workflowWithMemory = new Workflow({
  steps: [
    new Step({
      id: 'recall-context',
      execute: async (data) => {
        const context = await memory.recall({
          threadId: data.threadId,
          context: data.userMessage,
          limit: 5
        });
        
        return { ...data, context };
      }
    }),
    new Step({
      id: 'process-with-context',
      execute: async (data) => {
        // Use recalled context in processing
        const agent = mastra.getAgent('assistant');
        return agent.generate(data.userMessage, {
          context: data.context.messages
        });
      }
    })
  ]
});
```

## Performance Optimization

### Batch Operations

```typescript
// Batch save multiple messages
await memory.batchSaveMessages([
  { threadId: 'conv-1', messages: messages1 },
  { threadId: 'conv-2', messages: messages2 },
  { threadId: 'conv-3', messages: messages3 }
]);

// Batch queries
const results = await memory.batchQuery([
  { query: 'payment issues', threadId: 'conv-1' },
  { query: 'shipping problems', threadId: 'conv-2' }
]);
```

### Caching

```typescript
const memory = new MastraMemory({
  cache: {
    enabled: true,
    ttl: 300000, // 5 minutes
    maxSize: 1000 // entries
  }
});
```

## Monitoring and Analytics

```typescript
// Memory usage statistics
const stats = await memory.getStatistics();
console.log('Total threads:', stats.threadCount);
console.log('Total messages:', stats.messageCount);
console.log('Storage used:', stats.storageSize);

// Message analytics
const analytics = await memory.getAnalytics({
  threadId: 'conversation-123',
  timeRange: { from: lastWeek, to: now }
});
console.log('Message frequency:', analytics.messageFrequency);
console.log('Average response time:', analytics.avgResponseTime);
```

## Error Handling

```typescript
try {
  await memory.saveMessages({ threadId, messages });
} catch (error) {
  if (error.code === 'STORAGE_FULL') {
    // Handle storage capacity issues
    await memory.compressOldMessages();
    // Retry
    await memory.saveMessages({ threadId, messages });
  } else if (error.code === 'INVALID_MESSAGE_FORMAT') {
    // Handle validation errors
    const validatedMessages = validateAndSanitize(messages);
    await memory.saveMessages({ threadId, messages: validatedMessages });
  } else {
    throw error;
  }
}
```

## See Also

- [Agent API](../agents/index.md) - Using memory with agents
- [Storage API](../storage/index.md) - Storage backend configuration  
- [Conversation Patterns](../../03-patterns/conversation-patterns.md)

## Next Steps

- [Set up conversation flows](../../03-patterns/conversation-patterns.md)
- [Integrate with agents](../agents/index.md#memory-integration)
- [Configure storage backends](../storage/index.md)