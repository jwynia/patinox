# Error Handling Patterns

> Exception types, error codes, and recovery strategies in Mastra

## Overview

Mastra provides comprehensive error handling with typed exceptions, retry logic, and recovery strategies. Understanding these patterns helps build resilient AI applications that gracefully handle failures.

## Error Types

### Base Error Class

```typescript
class MastraError extends Error {
  code: string;
  details?: any;
  cause?: Error;
  
  constructor(message: string, code: string, details?: any) {
    super(message);
    this.code = code;
    this.details = details;
  }
}
```

### Specific Error Classes

```typescript
// Agent-related errors
class AgentError extends MastraError {}

// Workflow errors
class WorkflowError extends MastraError {}

// Storage errors
class StorageError extends MastraError {}

// Integration errors
class IntegrationError extends MastraError {}

// Validation errors
class ValidationError extends MastraError {
  errors: ZodError[];
}
```

## Common Error Codes

### Agent Errors
- `AGENT_NOT_FOUND` - Agent doesn't exist
- `MODEL_ERROR` - LLM provider error
- `TOOL_EXECUTION_FAILED` - Tool threw exception
- `RATE_LIMIT_EXCEEDED` - API rate limit hit
- `CONTEXT_LENGTH_EXCEEDED` - Too many tokens
- `INVALID_RESPONSE` - Malformed LLM response

### Workflow Errors
- `WORKFLOW_NOT_FOUND` - Workflow doesn't exist
- `STEP_FAILED` - Step execution failed
- `TIMEOUT` - Workflow exceeded time limit
- `SUSPENDED` - Workflow is suspended
- `INVALID_STATE` - Invalid state transition

### Storage Errors
- `CONNECTION_FAILED` - Database unreachable
- `TRANSACTION_FAILED` - Transaction rolled back
- `KEY_NOT_FOUND` - Requested key doesn't exist
- `QUOTA_EXCEEDED` - Storage limit reached

## Error Handling Strategies

### Try-Catch Pattern

```typescript
// Basic error handling
try {
  const result = await agent.generate(prompt);
  return result.text;
} catch (error) {
  if (error instanceof AgentError) {
    // Handle agent-specific errors
    console.error(`Agent error: ${error.code}`);
    
    if (error.code === 'RATE_LIMIT_EXCEEDED') {
      // Wait and retry
      await delay(1000);
      return agent.generate(prompt);
    }
  }
  
  // Re-throw unknown errors
  throw error;
}
```

### Error Boundaries in Workflows

```typescript
const workflow = new Workflow({
  steps: [
    new Step({
      id: 'risky-operation',
      execute: async (data) => {
        try {
          return await riskyOperation(data);
        } catch (error) {
          // Log but don't fail workflow
          console.error('Non-critical error:', error);
          return { success: false, error: error.message };
        }
      }
    })
  ],
  errorHandler: (error, context) => {
    // Global error handler
    logger.error('Workflow error', { error, context });
    // Decide whether to continue or abort
    if (error.code === 'CRITICAL') {
      throw error; // Abort workflow
    }
    // Continue with default value
    return { fallback: true };
  }
});
```

### Retry Logic

```typescript
// Built-in retry with exponential backoff
const result = await agent.generate(prompt, {
  maxRetries: 3,
  retryDelay: 1000, // Initial delay
  retryBackoff: 2,  // Exponential factor
});

// Custom retry logic
async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options: {
    maxAttempts?: number;
    initialDelay?: number;
    maxDelay?: number;
    factor?: number;
  } = {}
): Promise<T> {
  const {
    maxAttempts = 3,
    initialDelay = 1000,
    maxDelay = 30000,
    factor = 2
  } = options;
  
  let lastError: Error;
  
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      
      if (attempt === maxAttempts - 1) {
        throw error;
      }
      
      const delay = Math.min(
        initialDelay * Math.pow(factor, attempt),
        maxDelay
      );
      
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError!;
}

// Usage
const result = await retryWithBackoff(
  () => agent.generate(prompt),
  { maxAttempts: 5, initialDelay: 500 }
);
```

### Circuit Breaker Pattern

```typescript
class CircuitBreaker {
  private failures = 0;
  private lastFailTime?: Date;
  private state: 'closed' | 'open' | 'half-open' = 'closed';
  
  constructor(
    private threshold: number = 5,
    private timeout: number = 60000
  ) {}
  
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === 'open') {
      if (Date.now() - this.lastFailTime!.getTime() > this.timeout) {
        this.state = 'half-open';
      } else {
        throw new Error('Circuit breaker is open');
      }
    }
    
    try {
      const result = await fn();
      if (this.state === 'half-open') {
        this.state = 'closed';
        this.failures = 0;
      }
      return result;
    } catch (error) {
      this.failures++;
      this.lastFailTime = new Date();
      
      if (this.failures >= this.threshold) {
        this.state = 'open';
      }
      
      throw error;
    }
  }
}

// Usage
const breaker = new CircuitBreaker(3, 30000);

try {
  const result = await breaker.execute(
    () => externalApi.call()
  );
} catch (error) {
  // Handle circuit open or API error
}
```

## Validation Error Handling

```typescript
// Schema validation
const schema = z.object({
  name: z.string().min(1),
  age: z.number().positive()
});

try {
  const validated = schema.parse(input);
} catch (error) {
  if (error instanceof z.ZodError) {
    // Handle validation errors
    error.errors.forEach(err => {
      console.error(`Field ${err.path.join('.')}: ${err.message}`);
    });
  }
}

// Tool parameter validation
const tool = createTool({
  schema: z.object({
    query: z.string().min(1).max(100)
  }),
  executor: async (params) => {
    // params are guaranteed valid here
  }
});
```

## Async Error Handling

### Promise Rejection

```typescript
// Unhandled rejection handler
process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection:', reason);
  // Log to monitoring service
  monitoring.logError(reason);
});

// Async function error handling
async function safeExecute() {
  try {
    await riskyAsyncOperation();
  } catch (error) {
    // Always handle async errors
    handleError(error);
  }
}
```

### Stream Error Handling

```typescript
// Handle stream errors
const stream = await agent.stream(prompt);

try {
  for await (const chunk of stream) {
    process.stdout.write(chunk);
  }
} catch (error) {
  if (error.code === 'STREAM_INTERRUPTED') {
    console.error('Stream was interrupted');
  }
}

// With error event
stream.on('error', (error) => {
  console.error('Stream error:', error);
});
```

## Error Recovery Strategies

### Fallback Values

```typescript
async function getWithFallback<T>(
  primary: () => Promise<T>,
  fallback: T
): Promise<T> {
  try {
    return await primary();
  } catch (error) {
    console.warn('Using fallback due to error:', error);
    return fallback;
  }
}

// Usage
const result = await getWithFallback(
  () => agent.generate(prompt),
  { text: 'Sorry, I cannot process this request.' }
);
```

### Graceful Degradation

```typescript
async function processWithDegradation(data: any) {
  // Try advanced model first
  try {
    return await advancedAgent.generate(data);
  } catch (error) {
    console.warn('Falling back to basic model');
    
    // Fall back to simpler model
    try {
      return await basicAgent.generate(data);
    } catch (error) {
      console.error('All models failed');
      
      // Return static response
      return { text: 'Service temporarily unavailable' };
    }
  }
}
```

### Compensation Actions

```typescript
const workflow = new Workflow({
  steps: [
    new Step({
      id: 'create-resource',
      execute: async (data) => {
        const resource = await createResource(data);
        return { resourceId: resource.id };
      },
      compensate: async (data, result) => {
        // Cleanup on failure
        if (result.resourceId) {
          await deleteResource(result.resourceId);
        }
      }
    })
  ]
});
```

## Logging and Monitoring

```typescript
// Structured error logging
function logError(error: Error, context: any = {}) {
  const errorInfo = {
    message: error.message,
    stack: error.stack,
    code: error instanceof MastraError ? error.code : 'UNKNOWN',
    timestamp: new Date().toISOString(),
    ...context
  };
  
  // Log to different destinations based on severity
  if (error instanceof CriticalError) {
    alerting.send(errorInfo);
  }
  
  logger.error(errorInfo);
}

// Usage
try {
  await riskyOperation();
} catch (error) {
  logError(error, {
    userId: context.userId,
    operation: 'riskyOperation',
    params: sanitize(params)
  });
}
```

## Testing Error Scenarios

```typescript
// Test error handling
describe('Error handling', () => {
  it('should retry on rate limit', async () => {
    const agent = new Agent({ model: 'gpt-4' });
    
    // Mock rate limit error
    vi.spyOn(agent, 'generate')
      .mockRejectedValueOnce(new AgentError('Rate limited', 'RATE_LIMIT'))
      .mockResolvedValueOnce({ text: 'Success' });
    
    const result = await retryWithBackoff(
      () => agent.generate('test')
    );
    
    expect(result.text).toBe('Success');
    expect(agent.generate).toHaveBeenCalledTimes(2);
  });
});
```

## See Also

- [Async Patterns](./async-patterns.md)
- [Performance Patterns](./performance-patterns.md)
- [Testing Setup](../04-integration/testing-setup.md)

## Next Steps

- [Implement retry strategies](./performance-patterns.md)
- [Set up monitoring](../06-advanced/debugging.md)
- [Test error scenarios](../04-integration/testing-setup.md)