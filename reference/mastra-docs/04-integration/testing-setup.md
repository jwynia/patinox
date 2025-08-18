# Testing Setup

> How to test agents, workflows, and AI applications built with Mastra

## Overview

Testing AI applications requires special considerations for non-deterministic outputs, external API dependencies, and long-running processes. This guide covers testing strategies for Mastra components including agents, workflows, tools, memory systems, and integrations.

## Testing Philosophy

### Types of Tests

1. **Unit Tests** - Individual components in isolation
2. **Integration Tests** - Components working together
3. **Contract Tests** - API and interface compliance
4. **End-to-End Tests** - Complete user scenarios
5. **Performance Tests** - Response times and throughput
6. **Evaluation Tests** - AI output quality assessment

### AI-Specific Challenges

- **Non-deterministic outputs** - Same input may produce different results
- **External dependencies** - LLM providers, vector databases
- **Slow operations** - Generation and embedding can take time
- **Quality assessment** - Measuring "good" vs "bad" responses

## Test Environment Setup

### Basic Test Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    setupFiles: ['./tests/setup.ts'],
    testTimeout: 30000, // AI operations can be slow
    hookTimeout: 10000,
    include: ['**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*'],
      exclude: ['src/**/*.test.ts', 'src/test-utils/**']
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  }
});
```

### Test Environment Variables

```bash
# .env.test
NODE_ENV=test

# Test database
TEST_DATABASE_URL=postgresql://user:password@localhost:5432/mastra_test

# Mock API keys (or use real ones for integration tests)
OPENAI_API_KEY=test-key-openai
ANTHROPIC_API_KEY=test-key-anthropic

# Test vector store
TEST_PINECONE_API_KEY=test-pinecone-key
TEST_PINECONE_INDEX=test-index

# Enable test mode flags
MASTRA_TEST_MODE=true
MASTRA_MOCK_LLM=true
```

### Global Test Setup

```typescript
// tests/setup.ts
import { beforeAll, beforeEach, afterAll, afterEach } from 'vitest';
import { setupTestDatabase, cleanupTestDatabase } from './test-utils/database';
import { MockLLMProvider } from './test-utils/mock-llm';
import { TestConfig } from './test-utils/config';

// Global test configuration
beforeAll(async () => {
  // Setup test database
  await setupTestDatabase();
  
  // Configure mock providers
  TestConfig.enableMockMode();
  
  // Initialize test data
  await seedTestData();
});

afterAll(async () => {
  await cleanupTestDatabase();
});

beforeEach(async () => {
  // Clear test data between tests
  await clearTestData();
});

// Utility functions for tests
global.mockLLM = new MockLLMProvider();
global.testDb = TestConfig.getTestDatabase();
```

## Agent Testing

### Unit Testing Agents

```typescript
// tests/agent.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Agent } from '@mastra/core';
import { MockLLMProvider } from './test-utils/mock-llm';

describe('Agent', () => {
  let agent: Agent;
  let mockProvider: MockLLMProvider;

  beforeEach(() => {
    mockProvider = new MockLLMProvider();
    
    agent = new Agent({
      model: 'mock:gpt-4',
      provider: mockProvider,
      instructions: 'You are a helpful assistant'
    });
  });

  describe('generate', () => {
    it('should generate text response', async () => {
      // Arrange
      const prompt = 'Hello, how are you?';
      const expectedResponse = 'I am doing well, thank you!';
      
      mockProvider.mockResponse({
        text: expectedResponse,
        usage: { promptTokens: 10, completionTokens: 15, totalTokens: 25 }
      });

      // Act
      const result = await agent.generate(prompt);

      // Assert
      expect(result.text).toBe(expectedResponse);
      expect(result.usage.totalTokens).toBe(25);
      expect(mockProvider.getLastCall()).toMatchObject({
        prompt,
        model: 'mock:gpt-4'
      });
    });

    it('should handle tool calls', async () => {
      // Setup agent with tools
      const agentWithTools = new Agent({
        model: 'mock:gpt-4',
        provider: mockProvider,
        tools: {
          getCurrentTime: {
            description: 'Get current time',
            schema: z.object({}),
            executor: vi.fn().mockResolvedValue({ time: '2024-01-01T00:00:00Z' })
          }
        }
      });

      // Mock tool call response
      mockProvider.mockToolCall({
        toolName: 'getCurrentTime',
        args: {},
        result: { time: '2024-01-01T00:00:00Z' }
      });

      const result = await agentWithTools.generate('What time is it?');

      expect(result.toolCalls).toHaveLength(1);
      expect(result.toolCalls[0].toolName).toBe('getCurrentTime');
    });

    it('should handle errors gracefully', async () => {
      // Mock provider error
      mockProvider.mockError(new Error('Rate limit exceeded'));

      await expect(agent.generate('test')).rejects.toThrow('Rate limit exceeded');
    });
  });

  describe('stream', () => {
    it('should stream text chunks', async () => {
      const chunks = ['Hello', ' there', '!'];
      mockProvider.mockStream(chunks);

      const stream = await agent.stream('Hello');
      const receivedChunks: string[] = [];

      for await (const chunk of stream) {
        receivedChunks.push(chunk);
      }

      expect(receivedChunks).toEqual(chunks);
    });
  });

  describe('generateObject', () => {
    it('should generate structured output', async () => {
      const schema = z.object({
        name: z.string(),
        age: z.number()
      });

      const expectedObject = { name: 'John', age: 30 };
      mockProvider.mockStructuredResponse(expectedObject);

      const result = await agent.generateObject('Generate a person', schema);

      expect(result).toEqual(expectedObject);
    });

    it('should validate schema compliance', async () => {
      const schema = z.object({
        name: z.string(),
        age: z.number()
      });

      // Mock invalid response
      mockProvider.mockStructuredResponse({ name: 'John', age: 'thirty' });

      await expect(
        agent.generateObject('Generate a person', schema)
      ).rejects.toThrow();
    });
  });
});
```

### Integration Testing with Real LLMs

```typescript
// tests/agent.integration.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { Agent } from '@mastra/core';
import { createOpenAI } from '@ai-sdk/openai';

describe('Agent Integration Tests', () => {
  let agent: Agent;

  beforeEach(() => {
    // Use real provider for integration tests
    agent = new Agent({
      model: 'openai:gpt-3.5-turbo',
      provider: createOpenAI({
        apiKey: process.env.OPENAI_API_KEY
      }),
      instructions: 'You are a helpful assistant. Be concise.'
    });
  });

  it('should handle basic conversation', async () => {
    const result = await agent.generate('Say hello in one word');
    
    expect(result.text).toBeTruthy();
    expect(result.text.length).toBeGreaterThan(0);
    expect(result.usage.totalTokens).toBeGreaterThan(0);
  }, 15000); // Longer timeout for real API calls

  it('should maintain consistency with temperature 0', async () => {
    const prompt = 'What is 2+2? Answer with just the number.';
    
    const results = await Promise.all([
      agent.generate(prompt, { temperature: 0 }),
      agent.generate(prompt, { temperature: 0 }),
      agent.generate(prompt, { temperature: 0 })
    ]);

    // With temperature 0, results should be identical
    expect(results[0].text).toBe(results[1].text);
    expect(results[1].text).toBe(results[2].text);
  }, 30000);
});
```

## Workflow Testing

### Unit Testing Workflows

```typescript
// tests/workflow.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Workflow, Step } from '@mastra/core';

describe('Workflow', () => {
  let workflow: Workflow;

  beforeEach(() => {
    workflow = new Workflow({
      name: 'test-workflow',
      steps: [
        new Step({
          id: 'step1',
          execute: vi.fn().mockResolvedValue({ output: 'step1-result' })
        }),
        new Step({
          id: 'step2',
          execute: vi.fn().mockResolvedValue({ output: 'step2-result' })
        })
      ]
    });
  });

  describe('execution', () => {
    it('should execute steps in order', async () => {
      const { start } = workflow.createRun();
      const result = await start({ input: 'test-data' });

      expect(result.status).toBe('completed');
      expect(result.executedSteps).toHaveLength(2);
      expect(result.executedSteps[0].stepId).toBe('step1');
      expect(result.executedSteps[1].stepId).toBe('step2');
    });

    it('should handle step failures', async () => {
      // Mock step failure
      const failingWorkflow = new Workflow({
        name: 'failing-workflow',
        steps: [
          new Step({
            id: 'failing-step',
            execute: vi.fn().mockRejectedValue(new Error('Step failed'))
          })
        ]
      });

      const { start } = failingWorkflow.createRun();
      const result = await start({ input: 'test-data' });

      expect(result.status).toBe('failed');
      expect(result.error).toBeDefined();
    });

    it('should support suspend and resume', async () => {
      const suspendingWorkflow = new Workflow({
        name: 'suspending-workflow',
        steps: [
          new Step({
            id: 'suspend-step',
            execute: async (data, context) => {
              await context.suspend('waiting-for-input', { checkpointData: data });
              return { resumed: true };
            }
          })
        ]
      });

      const { start, resume } = suspendingWorkflow.createRun();
      
      // Start execution (should suspend)
      const suspendResult = await start({ input: 'test' });
      expect(suspendResult.status).toBe('suspended');
      expect(suspendResult.suspendReason).toBe('waiting-for-input');

      // Resume execution
      const resumeResult = await resume({ additionalData: 'resumed' });
      expect(resumeResult.status).toBe('completed');
    });
  });

  describe('error handling', () => {
    it('should retry failed steps', async () => {
      let attempts = 0;
      const retryWorkflow = new Workflow({
        name: 'retry-workflow',
        steps: [
          new Step({
            id: 'retry-step',
            retries: 2,
            execute: vi.fn().mockImplementation(() => {
              attempts++;
              if (attempts < 3) {
                throw new Error('Temporary failure');
              }
              return { success: true };
            })
          })
        ]
      });

      const { start } = retryWorkflow.createRun();
      const result = await start({ input: 'test' });

      expect(result.status).toBe('completed');
      expect(attempts).toBe(3);
    });

    it('should call error handler on failure', async () => {
      const errorHandler = vi.fn().mockResolvedValue({ recovered: true });
      
      const workflowWithHandler = new Workflow({
        name: 'error-handled-workflow',
        errorHandler,
        steps: [
          new Step({
            id: 'failing-step',
            execute: vi.fn().mockRejectedValue(new Error('Step error'))
          })
        ]
      });

      const { start } = workflowWithHandler.createRun();
      await start({ input: 'test' });

      expect(errorHandler).toHaveBeenCalledWith(
        expect.any(Error),
        expect.objectContaining({
          stepId: 'failing-step',
          runId: expect.any(String)
        })
      );
    });
  });
});
```

### End-to-End Workflow Tests

```typescript
// tests/workflow.e2e.test.ts
import { describe, it, expect } from 'vitest';
import { Workflow, Step } from '@mastra/core';
import { mastra } from '../src/mastra';

describe('Workflow E2E Tests', () => {
  it('should process order workflow end-to-end', async () => {
    const orderWorkflow = new Workflow({
      name: 'process-order',
      steps: [
        new Step({
          id: 'validate-order',
          execute: async (data) => {
            expect(data.orderId).toBeDefined();
            expect(data.items).toBeInstanceOf(Array);
            return { ...data, validated: true };
          }
        }),
        new Step({
          id: 'calculate-total',
          execute: async (data) => {
            const total = data.items.reduce((sum, item) => sum + item.price, 0);
            return { ...data, total };
          }
        }),
        new Step({
          id: 'process-payment',
          execute: async (data) => {
            // Mock payment processing
            if (data.total > 1000) {
              throw new Error('Payment declined - amount too high');
            }
            return { ...data, paymentId: 'pay-123' };
          }
        }),
        new Step({
          id: 'send-confirmation',
          execute: async (data) => {
            return { 
              ...data, 
              confirmationSent: true,
              message: `Order ${data.orderId} confirmed with payment ${data.paymentId}`
            };
          }
        })
      ]
    });

    const { start } = orderWorkflow.createRun();
    const result = await start({
      orderId: 'order-123',
      items: [
        { name: 'Widget', price: 10 },
        { name: 'Gadget', price: 20 }
      ]
    });

    expect(result.status).toBe('completed');
    expect(result.data.total).toBe(30);
    expect(result.data.paymentId).toBe('pay-123');
    expect(result.data.confirmationSent).toBe(true);
  });

  it('should handle agent integration in workflow', async () => {
    const agent = mastra.getAgent('assistant');
    
    const agentWorkflow = new Workflow({
      name: 'agent-workflow',
      steps: [
        new Step({
          id: 'analyze-text',
          execute: async (data) => {
            const result = await agent.generateObject(
              `Analyze sentiment of: "${data.text}"`,
              z.object({
                sentiment: z.enum(['positive', 'negative', 'neutral']),
                confidence: z.number()
              })
            );
            
            return { ...data, analysis: result };
          }
        }),
        new Step({
          id: 'generate-response',
          execute: async (data) => {
            const prompt = `Generate a ${data.analysis.sentiment} response to: "${data.text}"`;
            const response = await agent.generate(prompt);
            
            return { ...data, response: response.text };
          }
        })
      ]
    });

    const { start } = agentWorkflow.createRun();
    const result = await start({
      text: 'I love this product!'
    });

    expect(result.status).toBe('completed');
    expect(result.data.analysis.sentiment).toBe('positive');
    expect(result.data.response).toBeTruthy();
  }, 30000);
});
```

## Memory Testing

```typescript
// tests/memory.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { MastraMemory } from '@mastra/memory';
import { testDb } from './test-utils/database';

describe('Memory', () => {
  let memory: MastraMemory;

  beforeEach(async () => {
    memory = new MastraMemory({
      name: 'test-memory',
      storage: testDb
    });
    
    await memory.init();
  });

  afterEach(async () => {
    await testDb.clear();
  });

  describe('message storage', () => {
    it('should save and retrieve messages', async () => {
      const messages = [
        { role: 'user', content: 'Hello' },
        { role: 'assistant', content: 'Hi there!' }
      ];

      await memory.saveMessages({
        threadId: 'test-thread',
        messages
      });

      const retrieved = await memory.getMessages({
        threadId: 'test-thread'
      });

      expect(retrieved).toHaveLength(2);
      expect(retrieved[0].content).toBe('Hello');
      expect(retrieved[1].content).toBe('Hi there!');
    });

    it('should handle thread isolation', async () => {
      await memory.saveMessages({
        threadId: 'thread-1',
        messages: [{ role: 'user', content: 'Thread 1 message' }]
      });

      await memory.saveMessages({
        threadId: 'thread-2',
        messages: [{ role: 'user', content: 'Thread 2 message' }]
      });

      const thread1Messages = await memory.getMessages({ threadId: 'thread-1' });
      const thread2Messages = await memory.getMessages({ threadId: 'thread-2' });

      expect(thread1Messages).toHaveLength(1);
      expect(thread2Messages).toHaveLength(1);
      expect(thread1Messages[0].content).toBe('Thread 1 message');
      expect(thread2Messages[0].content).toBe('Thread 2 message');
    });
  });

  describe('semantic search', () => {
    it('should find relevant messages', async () => {
      // Setup test data
      await memory.saveMessages({
        threadId: 'search-test',
        messages: [
          { role: 'user', content: 'I need help with my payment' },
          { role: 'assistant', content: 'I can help you with billing issues' },
          { role: 'user', content: 'What is the weather today?' },
          { role: 'assistant', content: 'The weather is sunny' }
        ]
      });

      const results = await memory.query({
        query: 'payment problems',
        threadId: 'search-test',
        limit: 2
      });

      expect(results).toHaveLength(2);
      expect(results[0].message.content).toContain('payment');
    });
  });
});
```

## Tool Testing

```typescript
// tests/tools.test.ts
import { describe, it, expect, vi } from 'vitest';
import { createTool } from '@mastra/core';
import { z } from 'zod';

describe('Tools', () => {
  describe('createTool', () => {
    it('should validate input schema', async () => {
      const tool = createTool({
        name: 'calculator',
        description: 'Add two numbers',
        schema: z.object({
          a: z.number(),
          b: z.number()
        }),
        executor: async ({ a, b }) => ({ result: a + b })
      });

      const result = await tool.executor({ a: 5, b: 3 });
      expect(result.result).toBe(8);
    });

    it('should reject invalid input', async () => {
      const tool = createTool({
        name: 'calculator',
        schema: z.object({
          a: z.number(),
          b: z.number()
        }),
        executor: async ({ a, b }) => ({ result: a + b })
      });

      await expect(
        tool.executor({ a: 'invalid', b: 3 })
      ).rejects.toThrow();
    });

    it('should handle async operations', async () => {
      const mockAsyncOperation = vi.fn().mockResolvedValue('async result');
      
      const tool = createTool({
        name: 'async-tool',
        schema: z.object({ input: z.string() }),
        executor: async ({ input }) => {
          const result = await mockAsyncOperation(input);
          return { output: result };
        }
      });

      const result = await tool.executor({ input: 'test' });
      
      expect(mockAsyncOperation).toHaveBeenCalledWith('test');
      expect(result.output).toBe('async result');
    });
  });

  describe('tool integration with agents', () => {
    it('should execute tools correctly', async () => {
      const mockTool = createTool({
        name: 'weather',
        description: 'Get weather for a location',
        schema: z.object({ location: z.string() }),
        executor: vi.fn().mockResolvedValue({
          temperature: 72,
          condition: 'sunny'
        })
      });

      const agent = new Agent({
        model: 'mock:gpt-4',
        tools: { weather: mockTool }
      });

      // Mock agent to call tool
      mockProvider.mockToolCall({
        toolName: 'weather',
        args: { location: 'San Francisco' },
        result: { temperature: 72, condition: 'sunny' }
      });

      const result = await agent.generate('What is the weather in San Francisco?');

      expect(result.toolCalls).toHaveLength(1);
      expect(mockTool.executor).toHaveBeenCalledWith({ location: 'San Francisco' });
    });
  });
});
```

## Performance Testing

```typescript
// tests/performance.test.ts
import { describe, it, expect } from 'vitest';
import { Agent } from '@mastra/core';
import { performance } from 'perf_hooks';

describe('Performance Tests', () => {
  describe('Agent Response Times', () => {
    it('should respond within acceptable time limits', async () => {
      const agent = new Agent({
        model: 'openai:gpt-3.5-turbo',
        temperature: 0 // For consistent timing
      });

      const start = performance.now();
      await agent.generate('Hello');
      const duration = performance.now() - start;

      // Should respond within 10 seconds
      expect(duration).toBeLessThan(10000);
    }, 15000);

    it('should handle concurrent requests efficiently', async () => {
      const agent = new Agent({
        model: 'openai:gpt-3.5-turbo'
      });

      const requests = Array(5).fill(null).map((_, i) => 
        agent.generate(`Count to ${i + 1}`)
      );

      const start = performance.now();
      const results = await Promise.all(requests);
      const duration = performance.now() - start;

      expect(results).toHaveLength(5);
      results.forEach(result => {
        expect(result.text).toBeTruthy();
      });

      // Concurrent requests should be faster than sequential
      expect(duration).toBeLessThan(30000);
    }, 45000);
  });

  describe('Memory Performance', () => {
    it('should handle large message histories efficiently', async () => {
      const memory = new MastraMemory({
        name: 'perf-test',
        storage: testDb
      });

      // Create large message history
      const messages = Array(1000).fill(null).map((_, i) => ({
        role: i % 2 === 0 ? 'user' : 'assistant',
        content: `Message ${i}`
      }));

      const start = performance.now();
      
      await memory.saveMessages({
        threadId: 'large-thread',
        messages
      });

      const retrieved = await memory.getMessages({
        threadId: 'large-thread',
        limit: 50
      });

      const duration = performance.now() - start;

      expect(retrieved).toHaveLength(50);
      expect(duration).toBeLessThan(5000); // Should complete within 5 seconds
    });
  });
});
```

## Mock Utilities

```typescript
// tests/test-utils/mock-llm.ts
export class MockLLMProvider {
  private responses: any[] = [];
  private currentIndex = 0;
  private lastCall: any = null;

  mockResponse(response: any): void {
    this.responses.push(response);
  }

  mockStream(chunks: string[]): void {
    this.responses.push({
      type: 'stream',
      chunks
    });
  }

  mockToolCall(toolCall: any): void {
    this.responses.push({
      type: 'tool-call',
      ...toolCall
    });
  }

  mockError(error: Error): void {
    this.responses.push({
      type: 'error',
      error
    });
  }

  mockStructuredResponse(object: any): void {
    this.responses.push({
      type: 'structured',
      object
    });
  }

  async generate(prompt: string, options: any = {}): Promise<any> {
    this.lastCall = { prompt, options };
    
    const response = this.responses[this.currentIndex++];
    
    if (!response) {
      throw new Error('No mock response configured');
    }

    if (response.type === 'error') {
      throw response.error;
    }

    if (response.type === 'stream') {
      return this.createMockStream(response.chunks);
    }

    return response;
  }

  private createMockStream(chunks: string[]): AsyncIterable<string> {
    return {
      async *[Symbol.asyncIterator]() {
        for (const chunk of chunks) {
          yield chunk;
        }
      }
    };
  }

  getLastCall(): any {
    return this.lastCall;
  }

  reset(): void {
    this.responses = [];
    this.currentIndex = 0;
    this.lastCall = null;
  }
}
```

## Test Data Management

```typescript
// tests/test-utils/fixtures.ts
export const testFixtures = {
  messages: [
    { role: 'user', content: 'Hello' },
    { role: 'assistant', content: 'Hi there! How can I help you?' },
    { role: 'user', content: 'What is AI?' },
    { role: 'assistant', content: 'AI stands for Artificial Intelligence...' }
  ],

  workflows: {
    simple: {
      name: 'simple-workflow',
      steps: [
        { id: 'step1', execute: (data) => ({ ...data, step1: true }) },
        { id: 'step2', execute: (data) => ({ ...data, step2: true }) }
      ]
    }
  },

  agents: {
    basic: {
      model: 'mock:gpt-4',
      instructions: 'You are a helpful assistant'
    },
    
    withTools: {
      model: 'mock:gpt-4',
      tools: {
        calculator: {
          description: 'Add numbers',
          schema: z.object({ a: z.number(), b: z.number() }),
          executor: ({ a, b }) => ({ result: a + b })
        }
      }
    }
  }
};

export function createTestAgent(config = testFixtures.agents.basic) {
  return new Agent(config);
}

export function createTestWorkflow(config = testFixtures.workflows.simple) {
  return new Workflow(config);
}
```

## Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: mastra_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Run unit tests
        run: pnpm test:unit
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/mastra_test
      
      - name: Run integration tests
        run: pnpm test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/mastra_test
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          file: ./coverage/coverage-final.json
```

## See Also

- [Error Handling](../03-patterns/error-handling.md)
- [Performance Patterns](../03-patterns/performance-patterns.md)
- [Deployment Guide](./deployment.md)

## Next Steps

- [Set up CI/CD pipelines](./deployment.md#cicd)
- [Implement monitoring](../06-advanced/monitoring.md)
- [Add evaluation metrics](../02-api-reference/evals/index.md)