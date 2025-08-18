# Performance Patterns

> Caching, batching, lazy evaluation, and optimization strategies

## Overview

This guide covers performance optimization patterns for Mastra applications. Learn how to implement caching strategies, batch operations, manage token usage, optimize memory consumption, and scale applications effectively.

## Token Optimization

### Context Window Management

```typescript
// Intelligent context truncation
async function optimizeContext(
  messages: Message[], 
  maxTokens: number = 4000
): Promise<Message[]> {
  // Always keep system message
  const systemMessage = messages.find(m => m.role === 'system');
  const otherMessages = messages.filter(m => m.role !== 'system');
  
  // Start with recent messages
  let selectedMessages = otherMessages.slice(-10);
  let tokenCount = estimateTokens(selectedMessages);
  
  if (tokenCount <= maxTokens) {
    return systemMessage ? [systemMessage, ...selectedMessages] : selectedMessages;
  }
  
  // Use semantic relevance to select historical context
  const currentQuery = otherMessages[otherMessages.length - 1]?.content;
  const relevantHistory = await memory.query({
    query: currentQuery,
    limit: 5,
    threshold: 0.7
  });
  
  // Combine recent + relevant, respecting token limit
  const contextMessages = [
    ...relevantHistory.slice(0, 3), // Most relevant history
    ...selectedMessages.slice(-5)   // Most recent messages
  ];
  
  return truncateToTokenLimit(contextMessages, maxTokens);
}

// Token estimation utility
function estimateTokens(messages: Message[]): number {
  // Rough estimation: 1 token ≈ 4 characters
  return messages.reduce((total, msg) => {
    return total + Math.ceil(msg.content.length / 4);
  }, 0);
}
```

### Prompt Compression

```typescript
// Compress long prompts while preserving meaning
class PromptCompressor {
  async compress(prompt: string, maxLength: number): Promise<string> {
    if (prompt.length <= maxLength) return prompt;
    
    // Use summarization agent for compression
    const summarizer = mastra.getAgent('summarizer');
    
    const result = await summarizer.generate(
      `Compress this text to under ${maxLength} characters while preserving key information:\n\n${prompt}`,
      { maxTokens: Math.ceil(maxLength / 4) }
    );
    
    return result.text;
  }
  
  // Bullet point compression for structured data
  compressStructured(data: any[]): string {
    return data
      .map(item => `• ${item.key}: ${item.value}`)
      .join('\n');
  }
}
```

## Caching Strategies

### Response Caching

```typescript
// Multi-level cache implementation
class ResponseCache {
  private memoryCache = new Map<string, CacheEntry>();
  private redisCache?: Redis;
  
  constructor(redisUrl?: string) {
    if (redisUrl) {
      this.redisCache = new Redis(redisUrl);
    }
  }
  
  // Generate cache key from prompt and parameters
  generateKey(prompt: string, options: any = {}): string {
    const key = {
      prompt: this.normalizePrompt(prompt),
      model: options.model,
      temperature: options.temperature,
      maxTokens: options.maxTokens
    };
    return createHash('sha256').update(JSON.stringify(key)).digest('hex');
  }
  
  async get(key: string): Promise<any | null> {
    // L1: Memory cache
    const memoryResult = this.memoryCache.get(key);
    if (memoryResult && !this.isExpired(memoryResult)) {
      return memoryResult.data;
    }
    
    // L2: Redis cache
    if (this.redisCache) {
      const redisResult = await this.redisCache.get(key);
      if (redisResult) {
        const parsed = JSON.parse(redisResult);
        // Promote to memory cache
        this.memoryCache.set(key, {
          data: parsed,
          expires: Date.now() + 300000 // 5 minutes
        });
        return parsed;
      }
    }
    
    return null;
  }
  
  async set(key: string, data: any, ttl: number = 3600000): Promise<void> {
    const entry = {
      data,
      expires: Date.now() + ttl
    };
    
    // Store in memory cache
    this.memoryCache.set(key, entry);
    
    // Store in Redis with TTL
    if (this.redisCache) {
      await this.redisCache.setex(key, Math.floor(ttl / 1000), JSON.stringify(data));
    }
  }
  
  private normalizePrompt(prompt: string): string {
    return prompt.toLowerCase().trim().replace(/\s+/g, ' ');
  }
  
  private isExpired(entry: CacheEntry): boolean {
    return Date.now() > entry.expires;
  }
}

// Usage with agents
const cache = new ResponseCache(process.env.REDIS_URL);

async function cachedGenerate(prompt: string, options: any = {}) {
  const cacheKey = cache.generateKey(prompt, options);
  
  // Check cache first
  const cached = await cache.get(cacheKey);
  if (cached) {
    return { ...cached, fromCache: true };
  }
  
  // Generate if not cached
  const agent = mastra.getAgent('assistant');
  const result = await agent.generate(prompt, options);
  
  // Cache the result (don't cache errors)
  if (result.finishReason === 'stop') {
    await cache.set(cacheKey, result, 3600000); // 1 hour
  }
  
  return result;
}
```

### Embedding Caching

```typescript
// Cache embeddings to avoid redundant API calls
class EmbeddingCache {
  private cache = new Map<string, number[]>();
  
  async getEmbedding(text: string): Promise<number[]> {
    const normalized = this.normalizeText(text);
    
    // Check cache
    if (this.cache.has(normalized)) {
      return this.cache.get(normalized)!;
    }
    
    // Generate embedding
    const embedding = await generateEmbedding(text);
    
    // Cache result
    this.cache.set(normalized, embedding);
    
    return embedding;
  }
  
  private normalizeText(text: string): string {
    return text.toLowerCase().trim().replace(/\s+/g, ' ');
  }
  
  // Batch embedding generation
  async batchEmbeddings(texts: string[]): Promise<number[][]> {
    const results: number[][] = [];
    const toGenerate: { index: number; text: string }[] = [];
    
    // Check cache for each text
    for (let i = 0; i < texts.length; i++) {
      const cached = this.cache.get(this.normalizeText(texts[i]));
      if (cached) {
        results[i] = cached;
      } else {
        toGenerate.push({ index: i, text: texts[i] });
      }
    }
    
    // Generate embeddings for uncached texts
    if (toGenerate.length > 0) {
      const embeddings = await generateEmbeddings(toGenerate.map(t => t.text));
      
      for (let i = 0; i < toGenerate.length; i++) {
        const { index, text } = toGenerate[i];
        const embedding = embeddings[i];
        
        results[index] = embedding;
        this.cache.set(this.normalizeText(text), embedding);
      }
    }
    
    return results;
  }
}
```

## Batch Processing

### Message Batch Operations

```typescript
// Batch message processing for better throughput
class MessageBatcher {
  private pendingBatches = new Map<string, BatchJob>();
  private readonly batchSize = 10;
  private readonly batchTimeout = 1000; // 1 second
  
  async process(threadId: string, message: Message): Promise<ProcessResult> {
    return new Promise((resolve, reject) => {
      // Get or create batch for thread
      let batch = this.pendingBatches.get(threadId);
      if (!batch) {
        batch = {
          messages: [],
          resolvers: [],
          rejecters: [],
          timeout: setTimeout(() => this.executeBatch(threadId), this.batchTimeout)
        };
        this.pendingBatches.set(threadId, batch);
      }
      
      // Add to batch
      batch.messages.push(message);
      batch.resolvers.push(resolve);
      batch.rejecters.push(reject);
      
      // Execute if batch is full
      if (batch.messages.length >= this.batchSize) {
        clearTimeout(batch.timeout);
        this.executeBatch(threadId);
      }
    });
  }
  
  private async executeBatch(threadId: string): Promise<void> {
    const batch = this.pendingBatches.get(threadId);
    if (!batch) return;
    
    this.pendingBatches.delete(threadId);
    
    try {
      // Process entire batch at once
      const results = await this.processBatch(batch.messages);
      
      // Resolve all promises
      for (let i = 0; i < results.length; i++) {
        batch.resolvers[i](results[i]);
      }
    } catch (error) {
      // Reject all promises
      batch.rejecters.forEach(reject => reject(error));
    }
  }
  
  private async processBatch(messages: Message[]): Promise<ProcessResult[]> {
    // Combine messages into single context
    const combinedContext = messages.map(m => m.content).join('\n\n');
    
    const agent = mastra.getAgent('assistant');
    const result = await agent.generate(
      `Process these messages and provide responses:\n\n${combinedContext}`,
      { 
        maxTokens: messages.length * 200,
        output: z.array(z.object({
          response: z.string(),
          confidence: z.number()
        }))
      }
    );
    
    return result.object.map((item, index) => ({
      messageId: messages[index].id,
      response: item.response,
      confidence: item.confidence
    }));
  }
}
```

### Document Processing

```typescript
// Batch document processing for RAG
class DocumentProcessor {
  async processBatch(documents: Document[]): Promise<ProcessedDocument[]> {
    const results: ProcessedDocument[] = [];
    
    // Process in parallel batches of 5
    for (let i = 0; i < documents.length; i += 5) {
      const batch = documents.slice(i, i + 5);
      
      const batchResults = await Promise.all(
        batch.map(doc => this.processDocument(doc))
      );
      
      results.push(...batchResults);
      
      // Small delay to avoid overwhelming the system
      if (i + 5 < documents.length) {
        await delay(100);
      }
    }
    
    return results;
  }
  
  private async processDocument(document: Document): Promise<ProcessedDocument> {
    // Extract text
    const text = await this.extractText(document);
    
    // Chunk document
    const chunks = await this.chunkText(text, {
      size: 512,
      overlap: 50
    });
    
    // Generate embeddings in batch
    const embeddings = await this.embeddingCache.batchEmbeddings(chunks);
    
    return {
      id: document.id,
      chunks: chunks.map((chunk, index) => ({
        text: chunk,
        embedding: embeddings[index],
        metadata: {
          source: document.name,
          chunkIndex: index,
          totalChunks: chunks.length
        }
      }))
    };
  }
}
```

## Memory Management

### Memory Pool Pattern

```typescript
// Reuse expensive objects
class AgentPool {
  private available: Agent[] = [];
  private inUse = new Set<Agent>();
  private readonly maxSize = 10;
  
  async acquire(): Promise<Agent> {
    // Return available agent if exists
    if (this.available.length > 0) {
      const agent = this.available.pop()!;
      this.inUse.add(agent);
      return agent;
    }
    
    // Create new agent if under limit
    if (this.inUse.size < this.maxSize) {
      const agent = new Agent({
        model: 'gpt-4',
        tools: this.getSharedTools()
      });
      this.inUse.add(agent);
      return agent;
    }
    
    // Wait for agent to become available
    return new Promise((resolve) => {
      const checkAvailable = () => {
        if (this.available.length > 0) {
          const agent = this.available.pop()!;
          this.inUse.add(agent);
          resolve(agent);
        } else {
          setTimeout(checkAvailable, 10);
        }
      };
      checkAvailable();
    });
  }
  
  release(agent: Agent): void {
    this.inUse.delete(agent);
    
    // Reset agent state
    agent.clearContext?.();
    
    this.available.push(agent);
  }
  
  async withAgent<T>(fn: (agent: Agent) => Promise<T>): Promise<T> {
    const agent = await this.acquire();
    try {
      return await fn(agent);
    } finally {
      this.release(agent);
    }
  }
}
```

### Memory Leak Prevention

```typescript
// Automatic cleanup patterns
class ResourceManager {
  private resources = new Map<string, any>();
  private timers = new Map<string, NodeJS.Timeout>();
  
  register(id: string, resource: any, ttl: number = 300000): void {
    // Clean up existing resource
    this.cleanup(id);
    
    // Register new resource
    this.resources.set(id, resource);
    
    // Set cleanup timer
    const timer = setTimeout(() => this.cleanup(id), ttl);
    this.timers.set(id, timer);
  }
  
  get(id: string): any {
    return this.resources.get(id);
  }
  
  cleanup(id: string): void {
    const resource = this.resources.get(id);
    if (resource) {
      // Call cleanup method if exists
      if (typeof resource.cleanup === 'function') {
        resource.cleanup();
      }
      
      this.resources.delete(id);
    }
    
    const timer = this.timers.get(id);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(id);
    }
  }
  
  cleanupAll(): void {
    for (const id of this.resources.keys()) {
      this.cleanup(id);
    }
  }
}

// Auto-cleanup on process exit
const resourceManager = new ResourceManager();
process.on('exit', () => resourceManager.cleanupAll());
```

## Database Optimization

### Connection Pooling

```typescript
// Efficient database connection management
class DatabasePool {
  private pool: Pool;
  
  constructor(config: PoolConfig) {
    this.pool = new Pool({
      ...config,
      max: 20,              // Maximum connections
      idleTimeoutMillis: 30000,  // Close idle connections
      connectionTimeoutMillis: 2000, // Connection timeout
    });
    
    // Monitor pool health
    this.pool.on('error', (err) => {
      console.error('Database pool error:', err);
    });
  }
  
  async query(text: string, params?: any[]): Promise<QueryResult> {
    const start = Date.now();
    
    try {
      const result = await this.pool.query(text, params);
      const duration = Date.now() - start;
      
      // Log slow queries
      if (duration > 1000) {
        console.warn(`Slow query (${duration}ms): ${text}`);
      }
      
      return result;
    } catch (error) {
      console.error('Query error:', error);
      throw error;
    }
  }
  
  // Batch operations
  async batchInsert(table: string, records: any[]): Promise<void> {
    if (records.length === 0) return;
    
    const batchSize = 100;
    for (let i = 0; i < records.length; i += batchSize) {
      const batch = records.slice(i, i + batchSize);
      
      const values = batch.map((_, index) => 
        `($${index * Object.keys(batch[0]).length + 1}, $${index * Object.keys(batch[0]).length + 2}, ...)`
      ).join(', ');
      
      const params = batch.flatMap(record => Object.values(record));
      
      await this.query(
        `INSERT INTO ${table} (${Object.keys(batch[0]).join(', ')}) VALUES ${values}`,
        params
      );
    }
  }
}
```

### Query Optimization

```typescript
// Optimized queries for common operations
class OptimizedQueries {
  // Get messages with smart pagination
  async getMessages(threadId: string, options: GetMessagesOptions = {}) {
    const { limit = 50, cursor, includeEmbeddings = false } = options;
    
    const selectFields = includeEmbeddings 
      ? 'id, role, content, metadata, created_at, embedding'
      : 'id, role, content, metadata, created_at';
    
    let query = `
      SELECT ${selectFields}
      FROM messages 
      WHERE thread_id = $1
    `;
    
    const params = [threadId];
    
    if (cursor) {
      query += ` AND created_at < $2`;
      params.push(cursor);
    }
    
    query += ` ORDER BY created_at DESC LIMIT $${params.length + 1}`;
    params.push(limit);
    
    return this.db.query(query, params);
  }
  
  // Bulk update with conflict resolution
  async upsertMessages(messages: Message[]): Promise<void> {
    if (messages.length === 0) return;
    
    const query = `
      INSERT INTO messages (id, thread_id, role, content, metadata, created_at)
      VALUES ${messages.map((_, i) => `($${i * 6 + 1}, $${i * 6 + 2}, $${i * 6 + 3}, $${i * 6 + 4}, $${i * 6 + 5}, $${i * 6 + 6})`).join(', ')}
      ON CONFLICT (id) 
      DO UPDATE SET 
        content = EXCLUDED.content,
        metadata = EXCLUDED.metadata,
        updated_at = NOW()
    `;
    
    const params = messages.flatMap(msg => [
      msg.id, msg.threadId, msg.role, msg.content, 
      JSON.stringify(msg.metadata), msg.createdAt
    ]);
    
    await this.db.query(query, params);
  }
}
```

## Cold Start Optimization

### Serverless Patterns

```typescript
// Minimize cold start impact
class ServerlessOptimizer {
  private static instance: ServerlessOptimizer;
  private initialized = false;
  private initPromise?: Promise<void>;
  
  static getInstance(): ServerlessOptimizer {
    if (!this.instance) {
      this.instance = new ServerlessOptimizer();
    }
    return this.instance;
  }
  
  async init(): Promise<void> {
    if (this.initialized) return;
    
    if (this.initPromise) {
      return this.initPromise;
    }
    
    this.initPromise = this.performInit();
    await this.initPromise;
  }
  
  private async performInit(): Promise<void> {
    // Parallel initialization
    await Promise.all([
      this.initDatabase(),
      this.warmupModels(),
      this.preloadCache()
    ]);
    
    this.initialized = true;
  }
  
  private async initDatabase(): Promise<void> {
    // Create connection pool
    this.dbPool = new DatabasePool(config);
    
    // Warm up with dummy query
    await this.dbPool.query('SELECT 1');
  }
  
  private async warmupModels(): Promise<void> {
    // Pre-load model configurations
    const agent = mastra.getAgent('assistant');
    
    // Dummy generation to warm up
    await agent.generate('Hello', { maxTokens: 1 });
  }
  
  private async preloadCache(): Promise<void> {
    // Pre-load common responses
    const commonQueries = [
      'hello',
      'help',
      'how are you'
    ];
    
    for (const query of commonQueries) {
      const cached = await this.cache.get(query);
      if (!cached) {
        // Could pre-generate common responses
      }
    }
  }
}

// Use in serverless handler
export async function handler(event: any) {
  const optimizer = ServerlessOptimizer.getInstance();
  await optimizer.init();
  
  // Process request
  return processRequest(event);
}
```

## Cost Optimization

### Token Usage Monitoring

```typescript
class TokenUsageMonitor {
  private usage = new Map<string, UsageStats>();
  
  track(userId: string, usage: TokenUsage): void {
    const existing = this.usage.get(userId) || {
      promptTokens: 0,
      completionTokens: 0,
      totalTokens: 0,
      requestCount: 0,
      cost: 0
    };
    
    existing.promptTokens += usage.promptTokens;
    existing.completionTokens += usage.completionTokens;
    existing.totalTokens += usage.totalTokens;
    existing.requestCount += 1;
    existing.cost += this.calculateCost(usage);
    
    this.usage.set(userId, existing);
    
    // Alert on high usage
    if (existing.cost > 100) { // $100 threshold
      this.alertHighUsage(userId, existing);
    }
  }
  
  private calculateCost(usage: TokenUsage): number {
    // GPT-4 pricing (example)
    const inputCost = (usage.promptTokens / 1000) * 0.03;
    const outputCost = (usage.completionTokens / 1000) * 0.06;
    return inputCost + outputCost;
  }
  
  getUsageReport(userId: string): UsageReport {
    const stats = this.usage.get(userId);
    if (!stats) return { empty: true };
    
    return {
      ...stats,
      avgTokensPerRequest: stats.totalTokens / stats.requestCount,
      avgCostPerRequest: stats.cost / stats.requestCount
    };
  }
}
```

## Monitoring and Profiling

```typescript
// Performance monitoring
class PerformanceMonitor {
  private metrics = new Map<string, Metric[]>();
  
  time<T>(operation: string, fn: () => Promise<T>): Promise<T> {
    const start = performance.now();
    
    return fn().finally(() => {
      const duration = performance.now() - start;
      this.recordMetric(operation, duration);
    });
  }
  
  private recordMetric(operation: string, duration: number): void {
    const metrics = this.metrics.get(operation) || [];
    metrics.push({
      duration,
      timestamp: Date.now()
    });
    
    // Keep only recent metrics
    const oneHourAgo = Date.now() - 3600000;
    const filtered = metrics.filter(m => m.timestamp > oneHourAgo);
    
    this.metrics.set(operation, filtered);
  }
  
  getStats(operation: string): PerformanceStats {
    const metrics = this.metrics.get(operation) || [];
    if (metrics.length === 0) return { count: 0 };
    
    const durations = metrics.map(m => m.duration);
    
    return {
      count: metrics.length,
      avg: durations.reduce((a, b) => a + b) / durations.length,
      min: Math.min(...durations),
      max: Math.max(...durations),
      p95: this.percentile(durations, 0.95),
      p99: this.percentile(durations, 0.99)
    };
  }
  
  private percentile(values: number[], p: number): number {
    const sorted = values.sort((a, b) => a - b);
    const index = Math.ceil(sorted.length * p) - 1;
    return sorted[index];
  }
}

// Usage
const monitor = new PerformanceMonitor();

await monitor.time('agent-generation', async () => {
  return agent.generate(prompt);
});
```

## See Also

- [Error Handling](./error-handling.md)
- [Memory API](../02-api-reference/memory/index.md)
- [Performance Pitfalls](../05-gotchas/performance-pitfalls.md)

## Next Steps

- [Implement caching strategies](./caching-strategies.md)
- [Monitor production performance](../06-advanced/monitoring.md)
- [Scale your application](../06-advanced/scaling.md)