# Search Index

> Keyword and concept mapping to relevant documentation files

## Overview

This index helps agents quickly find relevant documentation by mapping keywords, concepts, and use cases to specific files. Use this for rapid lookup when you know what you're looking for.

## API Methods Quick Lookup

### Agent Methods
| Method | Purpose | File |
|--------|---------|------|
| `generate()` | Text generation | [Agent API](../02-api-reference/agents/index.md#generate) |
| `stream()` | Streaming text | [Agent API](../02-api-reference/agents/index.md#stream) |
| `generateObject()` | Structured output | [Agent API](../02-api-reference/agents/index.md#generateobject) |
| `streamObject()` | Streaming objects | [Agent API](../02-api-reference/agents/index.md#streamobject) |

### Workflow Methods
| Method | Purpose | File |
|--------|---------|------|
| `createRun()` | Start workflow | [Workflow API](../02-api-reference/workflows/index.md#createrun) |
| `start()` | Execute workflow | [Workflow API](../02-api-reference/workflows/index.md#start) |
| `suspend()` | Pause execution | [Workflow API](../02-api-reference/workflows/index.md#suspend) |
| `resume()` | Continue execution | [Workflow API](../02-api-reference/workflows/index.md#resume) |

### Memory Methods
| Method | Purpose | File |
|--------|---------|------|
| `saveMessages()` | Store conversation | [Memory API](../02-api-reference/memory/index.md#savemessages) |
| `getMessages()` | Retrieve history | [Memory API](../02-api-reference/memory/index.md#getmessages) |
| `query()` | Semantic search | [Memory API](../02-api-reference/memory/index.md#query) |

### RAG Methods
| Method | Purpose | File |
|--------|---------|------|
| `add()` | Add documents | [RAG API](../02-api-reference/rag/index.md#add) |
| `query()` | Search knowledge | [RAG API](../02-api-reference/rag/index.md#query) |
| `delete()` | Remove documents | [RAG API](../02-api-reference/rag/index.md#delete) |

### Tool Methods
| Method | Purpose | File |
|--------|---------|------|
| `createTool()` | Define tool | [Tools API](../02-api-reference/tools/index.md#createtool) |
| `validateTool()` | Validate schema | [Tools API](../02-api-reference/tools/index.md#validatetool) |

## Concept Mapping

### Core Concepts
- **Agent** → [Key Abstractions](../01-core-concepts/key-abstractions.md#agent)
- **Tool** → [Key Abstractions](../01-core-concepts/key-abstractions.md#tool)
- **Workflow** → [Key Abstractions](../01-core-concepts/key-abstractions.md#workflow)
- **Memory** → [Key Abstractions](../01-core-concepts/key-abstractions.md#memory)
- **Storage** → [Key Abstractions](../01-core-concepts/key-abstractions.md#storage)
- **Integration** → [Key Abstractions](../01-core-concepts/key-abstractions.md#integration)

### Architecture
- **Data Flow** → [Data Flow](../01-core-concepts/data-flow.md)
- **Component Relationships** → [Architecture Overview](../01-core-concepts/architecture-overview.md)
- **Design Philosophy** → [Mental Model](../01-core-concepts/mental-model.md)
- **System Design** → [Architecture Overview](../01-core-concepts/architecture-overview.md)

## Use Case Mapping

### Text Generation
- **Simple prompts** → [Minimal Example](../00-quick-start/minimal-example.md)
- **Structured output** → [Agent API](../02-api-reference/agents/index.md#generateobject)
- **Streaming responses** → [Agent API](../02-api-reference/agents/index.md#stream)

### Tool Usage
- **Creating tools** → [Tools API](../02-api-reference/tools/index.md)
- **Tool with agents** → [Minimal Example](../00-quick-start/minimal-example.md)
- **API integration** → [Integration Tutorials](../04-integration/api-integration.md)

### Conversation Management
- **Chat memory** → [Memory API](../02-api-reference/memory/index.md)
- **Thread management** → [Conversation Patterns](../03-patterns/conversation-patterns.md)
- **Context windows** → [Performance Patterns](../03-patterns/performance-patterns.md)

### Complex Workflows
- **Multi-step processes** → [Workflow API](../02-api-reference/workflows/index.md)
- **Conditional logic** → [Workflow Patterns](../03-patterns/workflow-patterns.md)
- **Error handling** → [Error Handling](../03-patterns/error-handling.md)

### Knowledge Bases
- **Document processing** → [RAG Tutorial](../04-integration/rag-tutorial.md)
- **Vector search** → [RAG API](../02-api-reference/rag/index.md)
- **Semantic recall** → [Memory API](../02-api-reference/memory/index.md)

## Technology Keywords

### LLM Providers
- **OpenAI** → [Installation](../00-quick-start/installation.md#llm-provider-setup)
- **Anthropic** → [Installation](../00-quick-start/installation.md#llm-provider-setup)
- **Google Gemini** → [Installation](../00-quick-start/installation.md#llm-provider-setup)

### Storage Systems
- **PostgreSQL** → [Storage Setup](../04-integration/storage-setup.md#postgresql)
- **Redis** → [Storage Setup](../04-integration/storage-setup.md#redis)
- **Pinecone** → [Vector Stores](../04-integration/vector-stores.md#pinecone)
- **ChromaDB** → [Vector Stores](../04-integration/vector-stores.md#chromadb)

### Deployment Platforms
- **Vercel** → [Deployment Guide](../04-integration/deployment.md#vercel)
- **Cloudflare** → [Deployment Guide](../04-integration/deployment.md#cloudflare)
- **Netlify** → [Deployment Guide](../04-integration/deployment.md#netlify)

## Problem-Solution Mapping

### Common Problems
| Problem | Solution File |
|---------|---------------|
| "Agent not responding" | [Troubleshooting](../05-gotchas/troubleshooting.md#agent-issues) |
| "Out of memory errors" | [Performance Pitfalls](../05-gotchas/performance-pitfalls.md#memory) |
| "Rate limiting" | [Error Handling](../03-patterns/error-handling.md#retry-logic) |
| "Context too long" | [Performance Patterns](../03-patterns/performance-patterns.md#context-management) |
| "Tool execution failed" | [Common Mistakes](../05-gotchas/common-mistakes.md#tool-mistakes) |
| "Workflow stuck" | [Workflow Debugging](../06-advanced/debugging.md#workflows) |

### Performance Issues
| Issue | Solution |
|-------|----------|
| Slow responses | [Performance Patterns](../03-patterns/performance-patterns.md) |
| High token usage | [Token Optimization](../03-patterns/performance-patterns.md#token-optimization) |
| Memory leaks | [Memory Management](../05-gotchas/performance-pitfalls.md#memory) |
| Database bottlenecks | [Storage Optimization](../03-patterns/performance-patterns.md#storage) |

## Feature Availability Matrix

### By Package
| Feature | @mastra/core | @mastra/memory | @mastra/rag | @mastra/cli |
|---------|--------------|----------------|-------------|-------------|
| Agents | ✅ | ❌ | ❌ | ✅ |
| Workflows | ✅ | ❌ | ❌ | ✅ |
| Tools | ✅ | ❌ | ❌ | ✅ |
| Memory | Basic | ✅ | ❌ | ❌ |
| RAG | ❌ | ❌ | ✅ | ❌ |
| Streaming | ✅ | ❌ | ❌ | ❌ |

### By Runtime
| Feature | Node.js | Edge | Browser | Mobile |
|---------|---------|------|---------|--------|
| Agents | ✅ | ✅ | Client SDK | Client SDK |
| Workflows | ✅ | ✅ | ❌ | ❌ |
| Storage | ✅ | Limited | ❌ | ❌ |
| Vector Search | ✅ | ✅ | ❌ | ❌ |

## Quick Commands Reference

### Installation
```bash
npx create-mastra@latest    # New project
pnpm add @mastra/core       # Add to existing
pnpm dev                    # Development server
```

### Common Imports
```typescript
import { Mastra, Agent, Workflow } from '@mastra/core';
import { createOpenAI } from '@ai-sdk/openai';
import { z } from 'zod';
```

### Environment Variables
```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
DATABASE_URL=postgresql://...
```

## Search Tips for Agents

1. **Start with use case** → Use case mapping above
2. **Find API method** → API methods table above  
3. **Check examples** → [Cheatsheet](../00-quick-start/cheatsheet.md)
4. **Debug issues** → Problem-solution mapping above
5. **Learn concepts** → [Learning Path](./learning-path.md)

## See Also

- [Decision Tree](./decision-tree.md) - If/then navigation logic
- [API Method Index](./api-method-index.md) - Complete method catalog  
- [Learning Path](./learning-path.md) - Structured reading guides