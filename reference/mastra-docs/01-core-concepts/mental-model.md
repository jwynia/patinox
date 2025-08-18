# Mental Model

> The paradigm and philosophy behind Mastra's design

## Overview

Mastra follows a "composition over configuration" philosophy, treating AI applications as orchestrated systems of specialized components. Think of it as building with intelligent LEGO blocks that handle specific concerns while working together seamlessly.

## Core Philosophy

### 1. Everything is a Component

Mastra treats all functionality as pluggable components:
- **Agents**: Intelligent decision makers
- **Tools**: Discrete capabilities
- **Workflows**: Process orchestrators
- **Memory**: Context providers
- **Storage**: Persistence layers

Each component has a single responsibility and well-defined interface.

### 2. Declarative Configuration

Define what you want, not how to build it:

```typescript
// Declare intent, not implementation
const mastra = new Mastra({
  agents: {
    researcher: {
      model: 'gpt-4',
      instructions: 'You research topics thoroughly',
      tools: ['search', 'summarize']
    }
  }
});

// Not imperative setup
// ❌ const agent = new Agent();
// ❌ agent.setModel('gpt-4');
// ❌ agent.addTool(searchTool);
```

### 3. Type-First Development

TypeScript isn't an afterthought—it's fundamental:
- Schema validation with Zod
- Type-safe tool parameters
- Compile-time guarantees
- Intellisense-driven development

### 4. Streaming by Default

Real-time interaction is the norm:
- Stream text as it's generated
- Progressive object updates
- Server-sent events
- Backpressure handling

## Conceptual Model

### The Orchestra Metaphor

Think of Mastra as an orchestra:
- **Mastra Instance**: The conductor
- **Agents**: Solo performers
- **Tools**: Individual instruments
- **Workflows**: The musical score
- **Memory**: The sheet music
- **Storage**: The music library

Each component plays its part, coordinated by the conductor, to create a harmonious application.

### The Factory Pattern

Components are created through factories:

```typescript
mastra.getAgent('name')     // Agent factory
workflow.createRun()         // Run factory
integration.createTool()     // Tool factory
```

This ensures proper initialization and dependency injection.

## Key Mental Shifts

### From Chains to Graphs

Traditional: Linear chains of prompts
```
Prompt 1 → Prompt 2 → Prompt 3 → Result
```

Mastra: Graph-based workflows
```
     ┌→ Step A →┐
Start → Decision → Merge → End
     └→ Step B →┘
```

### From Stateful to Stateless

Traditional: Stateful objects
```typescript
agent.memory.add(message);
agent.respond();  // Uses internal state
```

Mastra: Stateless with external state
```typescript
const messages = await memory.getMessages(threadId);
agent.generate(prompt, { messages });  // Pure function
```

### From Synchronous to Asynchronous

Traditional: Blocking calls
```typescript
const result = llm.complete(prompt);  // Blocks
```

Mastra: Async-first
```typescript
const stream = await agent.stream(prompt);  // Non-blocking
for await (const chunk of stream) { }
```

## Design Patterns

### Dependency Injection

All dependencies flow through Mastra:
```typescript
const mastra = new Mastra({
  storage: pgStorage,      // Inject storage
  memory: threadMemory,     // Inject memory
  providers: { openai }     // Inject providers
});
```

### Builder Pattern

Complex objects use builders:
```typescript
const tool = createTool()
  .name('search')
  .description('Search the web')
  .schema(z.object({ query: z.string() }))
  .executor(async ({ query }) => { /* ... */ })
  .build();
```

### Strategy Pattern

Swappable implementations:
```typescript
// Different storage strategies
const storage = process.env.DATABASE_URL 
  ? new PgStorage(config)
  : new RedisStorage(config);
```

### Observer Pattern

Event-driven updates:
```typescript
workflow.on('step:complete', (data) => {
  console.log(`Step ${data.stepId} completed`);
});
```

## Abstraction Layers

### Layer 1: Core Primitives
- Basic types and interfaces
- Runtime context
- Error types

### Layer 2: Domain Objects
- Agents
- Tools
- Workflows
- Memory

### Layer 3: Orchestration
- Mastra class
- Dependency injection
- Configuration management

### Layer 4: Application
- Your business logic
- Custom tools
- Specific workflows

## Thinking in Mastra

### Problem Decomposition

Break problems into:
1. **Data Sources**: What information is needed?
2. **Processing Steps**: How to transform data?
3. **Decision Points**: Where to branch logic?
4. **Side Effects**: What external actions?

### Component Selection

Choose components based on needs:
- **Need LLM reasoning?** → Use Agent
- **Need structured process?** → Use Workflow
- **Need external data?** → Use Tool/Integration
- **Need context?** → Use Memory
- **Need persistence?** → Use Storage

### Composition Strategy

Combine components effectively:
```typescript
// Agent with memory and tools
const agent = mastra.getAgent('assistant');

// Workflow using agent
const workflow = new Workflow({
  steps: [
    new Step({ 
      execute: async (data) => agent.generate(data.prompt)
    })
  ]
});

// Tool accessing integration
const tool = {
  executor: async (params) => {
    const github = mastra.getIntegration('github');
    return github.getIssues(params);
  }
};
```

## Common Patterns

### The Enrichment Pattern
```
Input → Validate → Enrich → Process → Output
         ↓          ↓         ↓
       Storage   External   Agent
                   API
```

### The Assistant Pattern
```
User Query → Memory Context → Agent → Tools → Response
                ↑                        ↓
                └────── Update ←─────────┘
```

### The Pipeline Pattern
```
Document → Chunk → Embed → Store → Index
            ↓        ↓       ↓        ↓
          Rules   Model   Vector   Search
```

## Anti-Patterns to Avoid

### ❌ Stateful Singletons
```typescript
// Bad: Global mutable state
global.agentMemory = [];
```

### ❌ Tight Coupling
```typescript
// Bad: Direct dependencies
class MyAgent {
  private db = new PostgreSQL();  // Coupled
}
```

### ❌ Synchronous Blocking
```typescript
// Bad: Blocks event loop
const result = fs.readFileSync('large.txt');
```

### ❌ Untyped Interfaces
```typescript
// Bad: No type safety
function processTool(params: any) { }
```

## See Also

- [Architecture Overview](./architecture-overview.md)
- [Key Abstractions](./key-abstractions.md)
- [Terminology](./terminology.md)

## Next Steps

- [Learn the terminology](./terminology.md)
- [Explore API patterns](../02-api-reference/api-conventions.md)
- [Study composition patterns](../03-patterns/composition-patterns.md)