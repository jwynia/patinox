# Agent API Reference

> Complete API documentation for the Agent module

## Overview

Agents are the primary interface for LLM interaction in Mastra. They combine language models with tools, memory, and structured output capabilities. Agents support both text generation and streaming, with automatic tool orchestration.

## Class: Agent

### Constructor

```typescript
new Agent(config: AgentConfig)
```

**Parameters**:
- `config: AgentConfig` - Agent configuration object

**Example**:
```typescript
const agent = new Agent({
  model: 'openai:gpt-4o',
  instructions: 'You are a helpful assistant',
  tools: { /* tool definitions */ },
  memory: { type: 'thread' }
});
```

## Methods

### generate

Generate a text response from the agent.

```typescript
agent.generate(
  prompt: string | PromptOptions,
  options?: GenerateOptions
): Promise<GenerateResult>
```

**Parameters**:
- `prompt` - Text prompt or structured prompt options
- `options` - Generation options (optional)

**Returns**: `Promise<GenerateResult>`

**Example**:
```typescript
// Simple prompt
const result = await agent.generate('What is the weather?');
console.log(result.text);

// With options
const result = await agent.generate('Analyze this data', {
  temperature: 0.5,
  maxTokens: 1000,
  output: z.object({ summary: z.string() })
});
```

### stream

Stream a response from the agent.

```typescript
agent.stream(
  prompt: string | PromptOptions,
  options?: StreamOptions
): Promise<ReadableStream>
```

**Parameters**:
- `prompt` - Text prompt or structured prompt options
- `options` - Streaming options (optional)

**Returns**: `Promise<ReadableStream>`

**Example**:
```typescript
const stream = await agent.stream('Tell me a story');
for await (const chunk of stream) {
  process.stdout.write(chunk);
}
```

### generateObject

Generate structured data matching a schema.

```typescript
agent.generateObject<T>(
  prompt: string,
  schema: ZodSchema<T>,
  options?: GenerateOptions
): Promise<T>
```

**Parameters**:
- `prompt` - Text prompt
- `schema` - Zod schema for output structure
- `options` - Generation options (optional)

**Returns**: `Promise<T>` - Validated object matching schema

**Example**:
```typescript
const schema = z.object({
  name: z.string(),
  age: z.number(),
  interests: z.array(z.string())
});

const person = await agent.generateObject(
  'Generate a fictional person profile',
  schema
);
```

### streamObject

Stream structured data updates.

```typescript
agent.streamObject<T>(
  prompt: string,
  schema: ZodSchema<T>,
  options?: StreamOptions
): Promise<AsyncIterable<Partial<T>>>
```

**Parameters**:
- `prompt` - Text prompt
- `schema` - Zod schema for output structure
- `options` - Streaming options (optional)

**Returns**: `Promise<AsyncIterable<Partial<T>>>`

**Example**:
```typescript
const stream = await agent.streamObject(prompt, schema);
for await (const partial of stream) {
  console.log('Update:', partial);
}
```

## Configuration

### AgentConfig

```typescript
interface AgentConfig {
  model: string;
  instructions?: string;
  tools?: Record<string, Tool> | string[];
  memory?: MemoryConfig;
  temperature?: number;
  maxTokens?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  stopSequences?: string[];
  inputProcessors?: InputProcessor[];
  outputProcessors?: OutputProcessor[];
}
```

### PromptOptions

```typescript
interface PromptOptions {
  system?: string;
  prompt?: string;
  messages?: CoreMessage[];
  threadId?: string;
  resourceId?: string;
}
```

### GenerateOptions

```typescript
interface GenerateOptions {
  temperature?: number;
  maxTokens?: number;
  output?: ZodSchema;
  tools?: Record<string, Tool>;
  toolChoice?: 'auto' | 'none' | string;
  abortSignal?: AbortSignal;
  headers?: Record<string, string>;
  maxRetries?: number;
  onStepFinish?: (event: StepFinishEvent) => void;
  onFinish?: (event: FinishEvent) => void;
}
```

### StreamOptions

```typescript
interface StreamOptions extends GenerateOptions {
  onChunk?: (chunk: StreamChunk) => void;
  onToolCall?: (toolCall: ToolCall) => void;
}
```

## Return Types

### GenerateResult

```typescript
interface GenerateResult {
  text: string;
  object?: any;
  usage: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  finishReason: 'stop' | 'length' | 'tool-calls' | 'error';
  toolCalls?: ToolCall[];
  warnings?: string[];
  providerMetadata?: Record<string, any>;
}
```

### ToolCall

```typescript
interface ToolCall {
  toolCallId: string;
  toolName: string;
  args: any;
  result?: any;
}
```

## Tool Integration

### Defining Tools

```typescript
const tools = {
  getCurrentWeather: {
    description: 'Get current weather for a location',
    schema: z.object({
      location: z.string(),
      unit: z.enum(['celsius', 'fahrenheit']).optional()
    }),
    executor: async ({ location, unit }) => {
      // Implementation
      return { temperature: 72, condition: 'sunny' };
    }
  }
};

const agent = new Agent({
  model: 'gpt-4',
  tools
});
```

### Tool Choice

```typescript
// Let model decide
await agent.generate(prompt, { toolChoice: 'auto' });

// Disable tools
await agent.generate(prompt, { toolChoice: 'none' });

// Force specific tool
await agent.generate(prompt, { toolChoice: 'getCurrentWeather' });
```

## Memory Integration

### Thread-Based Memory

```typescript
const agent = new Agent({
  model: 'gpt-4',
  memory: {
    type: 'thread',
    threadId: 'conversation-123'
  }
});

// Messages automatically saved to thread
await agent.generate('Hello');
await agent.generate('Follow up question');
```

### Custom Memory

```typescript
const agent = new Agent({
  model: 'gpt-4',
  memory: customMemoryInstance
});
```

## Input/Output Processing

### Input Processors

```typescript
const agent = new Agent({
  model: 'gpt-4',
  inputProcessors: [
    new LanguageDetector(),
    new PIIDetector(),
    new PromptInjectionDetector()
  ]
});
```

### Output Processors

```typescript
const agent = new Agent({
  model: 'gpt-4',
  outputProcessors: [
    new ModerationFilter(),
    new StructuredOutputValidator(schema)
  ]
});
```

## Error Handling

```typescript
try {
  const result = await agent.generate(prompt);
} catch (error) {
  if (error instanceof AgentError) {
    console.error('Agent error:', error.code, error.message);
  } else if (error instanceof ValidationError) {
    console.error('Validation error:', error.errors);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Events

```typescript
agent.on('generate:start', (event) => {
  console.log('Generation started');
});

agent.on('tool:start', (event) => {
  console.log(`Calling tool: ${event.toolName}`);
});

agent.on('tool:complete', (event) => {
  console.log(`Tool result:`, event.result);
});

agent.on('generate:complete', (event) => {
  console.log('Generation complete');
});
```

## Advanced Usage

### Multi-Agent Conversation

```typescript
const researcher = mastra.getAgent('researcher');
const writer = mastra.getAgent('writer');

const research = await researcher.generate('Research topic X');
const article = await writer.generate(`Write article based on: ${research.text}`);
```

### Streaming with Tool Calls

```typescript
const stream = await agent.stream(prompt);
for await (const chunk of stream) {
  if (chunk.type === 'text') {
    process.stdout.write(chunk.content);
  } else if (chunk.type === 'tool-call') {
    console.log('Tool called:', chunk.toolName);
  }
}
```

## See Also

- [Tool API](../tools/index.md)
- [Memory API](../memory/index.md)
- [Streaming Patterns](../../03-patterns/async-patterns.md)

## Next Steps

- [Configure tools for agents](../tools/index.md)
- [Set up memory systems](../memory/index.md)
- [Learn streaming patterns](../../03-patterns/async-patterns.md)