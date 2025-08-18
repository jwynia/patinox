# Workflow API Reference

> Complete API documentation for Workflows and Steps

## Overview

Workflows are durable, graph-based state machines that orchestrate multi-step processes. They support branching, loops, suspend/resume capabilities, error handling, and built-in telemetry. Workflows maintain state across executions and can be paused and resumed with external input.

## Class: Workflow

### Constructor

```typescript
new Workflow<TData>(config: WorkflowConfig<TData>)
```

**Parameters**:
- `config: WorkflowConfig<TData>` - Workflow configuration object

**Example**:
```typescript
const workflow = new Workflow({
  name: 'processOrder',
  version: '1.0.0',
  steps: [
    new Step({ id: 'validate', execute: validateOrder }),
    new Step({ id: 'process', execute: processPayment }),
    new Step({ id: 'fulfill', execute: fulfillOrder })
  ],
  errorHandler: handleError,
  timeout: 300000 // 5 minutes
});
```

## Methods

### createRun

Create a new workflow execution instance.

```typescript
workflow.createRun(options?: RunOptions): WorkflowRun<TData>
```

**Parameters**:
- `options?: RunOptions` - Optional run configuration

**Returns**: `WorkflowRun<TData>` - Execution instance

**Example**:
```typescript
const { start, suspend, resume, getStatus } = workflow.createRun({
  runId: 'order-123',
  context: { userId: 'user-456' }
});
```

### getStatus

Get the current status of a workflow run.

```typescript
workflow.getStatus(runId: string): WorkflowStatus
```

**Parameters**:
- `runId: string` - Unique run identifier

**Returns**: `WorkflowStatus` - Current execution status

**Example**:
```typescript
const status = workflow.getStatus('order-123');
console.log(status.phase); // 'running' | 'completed' | 'failed' | 'suspended'
```

## Class: WorkflowRun

### start

Begin workflow execution.

```typescript
start(triggerData?: TData): Promise<WorkflowResult<TData>>
```

**Parameters**:
- `triggerData?: TData` - Initial data for the workflow

**Returns**: `Promise<WorkflowResult<TData>>` - Execution result

**Example**:
```typescript
const result = await start({
  orderId: 'order-123',
  customerId: 'customer-456',
  items: [{ id: 'item-1', quantity: 2 }]
});

if (result.status === 'completed') {
  console.log('Order processed:', result.data);
} else if (result.status === 'suspended') {
  console.log('Waiting for:', result.suspendReason);
}
```

### suspend

Pause workflow execution at current step.

```typescript
suspend(reason?: string, data?: any): Promise<SuspendResult>
```

**Parameters**:
- `reason?: string` - Reason for suspension
- `data?: any` - Data to store during suspension

**Returns**: `Promise<SuspendResult>` - Suspension details

**Example**:
```typescript
// In a step that needs human approval
const suspendResult = await suspend('human_approval_required', {
  approvalRequest: {
    amount: order.total,
    requestedBy: order.customerId,
    timestamp: Date.now()
  }
});

// Store suspend data for later resume
await storage.set(`suspend:${runId}`, suspendResult);
```

### resume

Continue suspended workflow execution.

```typescript
resume(resumeData?: any): Promise<WorkflowResult<TData>>
```

**Parameters**:
- `resumeData?: any` - Data to merge when resuming

**Returns**: `Promise<WorkflowResult<TData>>` - Execution result

**Example**:
```typescript
// After approval is received
const approvalData = {
  approved: true,
  approvedBy: 'manager-123',
  approvedAt: Date.now()
};

const result = await resume(approvalData);
```

## Class: Step

### Constructor

```typescript
new Step<TInput, TOutput>(config: StepConfig<TInput, TOutput>)
```

**Parameters**:
- `config: StepConfig<TInput, TOutput>` - Step configuration

**Example**:
```typescript
const validateStep = new Step({
  id: 'validate-order',
  schema: z.object({
    orderId: z.string(),
    items: z.array(z.object({
      id: z.string(),
      quantity: z.number().positive()
    }))
  }),
  execute: async (data) => {
    // Validation logic
    if (!data.items.length) {
      throw new Error('Order must contain items');
    }
    
    return {
      ...data,
      validated: true,
      validatedAt: Date.now()
    };
  },
  onError: async (error, data) => {
    // Error handling logic
    return { error: error.message, recoverable: true };
  },
  timeout: 30000,
  retries: 3
});
```

## Configuration Types

### WorkflowConfig

```typescript
interface WorkflowConfig<TData> {
  name: string;
  version?: string;
  description?: string;
  steps: Step<any, any>[];
  errorHandler?: (error: Error, context: WorkflowContext) => Promise<any>;
  timeout?: number;
  retries?: number;
  maxConcurrency?: number;
  tags?: string[];
  metadata?: Record<string, any>;
}
```

### StepConfig

```typescript
interface StepConfig<TInput, TOutput> {
  id: string;
  name?: string;
  description?: string;
  schema?: ZodSchema<TInput>;
  execute: ExecuteFunction<TInput, TOutput>;
  onError?: ErrorHandler<TInput>;
  timeout?: number;
  retries?: number;
  condition?: (data: TInput) => boolean | Promise<boolean>;
  compensate?: CompensateFunction<TInput, TOutput>;
}
```

### ExecuteFunction

```typescript
type ExecuteFunction<TInput, TOutput> = (
  data: TInput,
  context: StepContext
) => Promise<TOutput> | TOutput;
```

**Context Properties**:
- `stepId: string` - Current step identifier
- `runId: string` - Workflow run identifier
- `attempt: number` - Current retry attempt
- `previousSteps: StepResult[]` - Results from previous steps
- `metadata: Record<string, any>` - Step metadata

## Return Types

### WorkflowResult

```typescript
interface WorkflowResult<TData> {
  status: 'completed' | 'failed' | 'suspended';
  data?: TData;
  error?: Error;
  suspendReason?: string;
  suspendData?: any;
  executedSteps: StepResult[];
  duration: number;
  startedAt: Date;
  endedAt?: Date;
}
```

### StepResult

```typescript
interface StepResult<TInput, TOutput> {
  stepId: string;
  status: 'success' | 'failed' | 'skipped' | 'suspended';
  input: TInput;
  output?: TOutput;
  error?: Error;
  duration: number;
  attempts: number;
  startedAt: Date;
  endedAt?: Date;
}
```

### WorkflowStatus

```typescript
interface WorkflowStatus {
  runId: string;
  phase: 'running' | 'completed' | 'failed' | 'suspended';
  currentStep?: string;
  progress: {
    completed: number;
    total: number;
    percentage: number;
  };
  duration: number;
  lastActivity: Date;
}
```

## Advanced Patterns

### Conditional Steps

```typescript
const conditionalStep = new Step({
  id: 'premium-processing',
  condition: (data) => data.customer.isPremium,
  execute: async (data) => {
    // Only runs for premium customers
    return expeditedProcessing(data);
  }
});
```

### Parallel Execution

```typescript
const parallelWorkflow = new Workflow({
  name: 'parallel-processing',
  steps: [
    new Step({ id: 'init', execute: initializeData }),
    // These steps run in parallel
    new Step({ 
      id: 'process-images',
      execute: processImages,
      dependsOn: ['init']
    }),
    new Step({ 
      id: 'process-text',
      execute: processText,
      dependsOn: ['init']
    }),
    // This waits for both parallel steps
    new Step({ 
      id: 'combine',
      execute: combineResults,
      dependsOn: ['process-images', 'process-text']
    })
  ]
});
```

### Loop Patterns

```typescript
const loopStep = new Step({
  id: 'retry-until-success',
  execute: async (data, context) => {
    const result = await attemptOperation(data);
    
    if (!result.success && context.attempt < 5) {
      // Continue looping
      return { next: 'retry-until-success', ...result };
    }
    
    return result;
  }
});
```

### Error Recovery

```typescript
const stepWithRecovery = new Step({
  id: 'risky-operation',
  execute: async (data) => {
    return await riskyApiCall(data);
  },
  onError: async (error, data) => {
    if (error.code === 'TEMPORARY_FAILURE') {
      // Retry with delay
      await delay(5000);
      return { retry: true };
    }
    
    // Use fallback
    return await fallbackOperation(data);
  }
});
```

### Compensation (Saga Pattern)

```typescript
const transactionalStep = new Step({
  id: 'charge-payment',
  execute: async (data) => {
    const charge = await paymentProvider.charge(data.paymentMethod, data.amount);
    return { ...data, chargeId: charge.id };
  },
  compensate: async (data, result) => {
    // Rollback on workflow failure
    if (result.chargeId) {
      await paymentProvider.refund(result.chargeId);
    }
  }
});
```

## Integration with Agents

```typescript
const agentStep = new Step({
  id: 'analyze-sentiment',
  execute: async (data, context) => {
    const agent = context.mastra.getAgent('sentiment-analyzer');
    
    const result = await agent.generateObject(
      `Analyze sentiment: ${data.text}`,
      z.object({
        sentiment: z.enum(['positive', 'negative', 'neutral']),
        confidence: z.number().min(0).max(1)
      })
    );
    
    return {
      ...data,
      sentiment: result.object
    };
  }
});
```

## Events and Monitoring

```typescript
// Workflow events
workflow.on('run:start', (event) => {
  console.log(`Workflow ${event.workflowName} started:`, event.runId);
});

workflow.on('step:complete', (event) => {
  console.log(`Step ${event.stepId} completed in ${event.duration}ms`);
});

workflow.on('run:suspend', (event) => {
  console.log(`Workflow suspended: ${event.reason}`);
  // Notify external systems
  notificationService.send(event);
});

workflow.on('run:complete', (event) => {
  console.log(`Workflow completed with status: ${event.status}`);
});
```

## Error Handling

```typescript
try {
  const result = await start(triggerData);
  
  if (result.status === 'failed') {
    console.error('Workflow failed:', result.error);
    
    // Check if recoverable
    if (result.error.code === 'RECOVERABLE') {
      // Attempt recovery
      const recoveryResult = await resume(recoveryData);
    }
  }
} catch (error) {
  console.error('Workflow execution error:', error);
}
```

## Performance Considerations

### Workflow Optimization

```typescript
const optimizedWorkflow = new Workflow({
  name: 'optimized-processing',
  maxConcurrency: 10, // Limit parallel steps
  steps: [
    new Step({
      id: 'batch-process',
      execute: async (data) => {
        // Process in batches to avoid memory issues
        const batches = chunk(data.items, 100);
        const results = [];
        
        for (const batch of batches) {
          const batchResult = await processBatch(batch);
          results.push(...batchResult);
        }
        
        return { ...data, results };
      }
    })
  ]
});
```

## See Also

- [Agent API](../agents/index.md) - Using agents in workflows
- [Error Handling Patterns](../../03-patterns/error-handling.md)
- [Workflow Patterns](../../03-patterns/workflow-patterns.md)

## Next Steps

- [Build complex workflows](../../03-patterns/workflow-patterns.md)
- [Integrate with agents](../../03-patterns/composition-patterns.md)
- [Handle errors gracefully](../../03-patterns/error-handling.md)