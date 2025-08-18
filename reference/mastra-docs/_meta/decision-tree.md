# Decision Tree

> If you want to do X, read Y - Navigation logic for agents

## Overview

This decision tree helps agents navigate directly to relevant documentation based on their specific goals. Instead of reading everything, use this guide to find exactly what you need.

## What Do You Want to Build?

### 🤖 Simple AI Agent
**Goal**: Create an agent that responds to prompts

**Read This**:
1. [Minimal Example](../00-quick-start/minimal-example.md) ← Start here
2. [Agent API](../02-api-reference/agents/index.md)
3. [Common Mistakes](../05-gotchas/common-mistakes.md#agent-mistakes)

**Skip**: Workflows, RAG, complex patterns

### 🛠️ Agent with Tools
**Goal**: Agent that can call functions/APIs

**Read This**:
1. [Agent API - Tool Integration](../02-api-reference/agents/index.md#tool-integration)
2. [Tools API](../02-api-reference/tools/index.md)
3. [Tool Patterns](../03-patterns/tool-patterns.md)

**Then Maybe**: Error handling, API integrations

### 💬 Chatbot with Memory
**Goal**: Conversational agent that remembers context

**Read This**:
1. [Memory API](../02-api-reference/memory/index.md)
2. [Conversation Patterns](../03-patterns/conversation-patterns.md)
3. [Chatbot Tutorial](../04-integration/chatbot-tutorial.md)

**Key Concept**: Thread management

### 📚 Knowledge-Base Agent (RAG)
**Goal**: Agent that answers questions from documents

**Read This**:
1. [RAG Tutorial](../04-integration/rag-tutorial.md) ← Complete guide
2. [RAG API](../02-api-reference/rag/index.md)
3. [Vector Store Setup](../04-integration/vector-stores.md)

**Prerequisites**: Understanding of embeddings

### 🔄 Multi-Step Automation
**Goal**: Complex workflows with branching logic

**Read This**:
1. [Workflow API](../02-api-reference/workflows/index.md)
2. [Workflow Patterns](../03-patterns/workflow-patterns.md)
3. [Automation Tutorial](../04-integration/automation-tutorial.md)

**Key Concepts**: Steps, suspend/resume, error recovery

### 🌐 Production Application
**Goal**: Deploy to production with monitoring

**Read This**:
1. [Production Deployment](../04-integration/deployment.md)
2. [Performance Patterns](../03-patterns/performance-patterns.md)
3. [Monitoring Setup](../06-advanced/monitoring.md)
4. [Security Considerations](../05-gotchas/security-considerations.md)

## What's Your Experience Level?

### 🆕 New to AI Development
**Path**: Guided learning with explanations

1. [Prerequisites](../00-quick-start/prerequisites.md)
2. [Mental Model](../01-core-concepts/mental-model.md)
3. [Minimal Example](../00-quick-start/minimal-example.md)
4. [Learning Path - Complete Beginner](../meta/learning-path.md#complete-beginner-2-3-hours)

### 🧠 Familiar with LLMs
**Path**: Focus on Mastra specifics

1. [Architecture Overview](../01-core-concepts/architecture-overview.md)
2. [Agent API](../02-api-reference/agents/index.md)
3. [Learning Path - AI Developer](../meta/learning-path.md#ai-developer-1-2-hours)

### ⚡ Want Quick Results
**Path**: Minimal reading, maximum doing

1. [Cheatsheet](../00-quick-start/cheatsheet.md) ← Copy-paste recipes
2. [Minimal Example](../00-quick-start/minimal-example.md)
3. [Common Mistakes](../05-gotchas/common-mistakes.md) ← Avoid pitfalls

## What Problem Are You Solving?

### 🐛 Something's Broken
**Issue**: Error messages, unexpected behavior

**Start Here**:
1. [Troubleshooting Guide](../05-gotchas/troubleshooting.md)
2. [Common Mistakes](../05-gotchas/common-mistakes.md)
3. [Error Handling Patterns](../03-patterns/error-handling.md)

**By Error Type**:
- Agent not responding → [Agent Issues](../05-gotchas/troubleshooting.md#agent-issues)
- Out of memory → [Performance Pitfalls](../05-gotchas/performance-pitfalls.md#memory)
- Rate limiting → [Error Handling](../03-patterns/error-handling.md#retry-logic)
- Tool failures → [Tool Debugging](../05-gotchas/troubleshooting.md#tool-issues)

### 🚀 Performance Issues
**Issue**: Slow responses, high costs, resource usage

**Read This**:
1. [Performance Pitfalls](../05-gotchas/performance-pitfalls.md)
2. [Performance Patterns](../03-patterns/performance-patterns.md)
3. [Optimization Guide](../06-advanced/optimization.md)

**By Issue**:
- Slow generation → [Token optimization](../03-patterns/performance-patterns.md#token-optimization)
- High costs → [Cost optimization](../03-patterns/performance-patterns.md#cost-optimization)
- Memory leaks → [Memory management](../05-gotchas/performance-pitfalls.md#memory)

### 🔒 Security Concerns
**Issue**: Input validation, data safety, auth

**Read This**:
1. [Security Considerations](../05-gotchas/security-considerations.md)
2. [Input Validation](../03-patterns/error-handling.md#validation-error-handling)
3. [Authentication Setup](../04-integration/auth-setup.md)

## What Technology Stack?

### Next.js Application
**Read This**:
1. [Next.js Integration](../04-integration/frameworks/nextjs.md)
2. [Deployment - Vercel](../04-integration/deployment.md#vercel)
3. [Client-Side Patterns](../03-patterns/client-patterns.md)

### Express/Node.js API
**Read This**:
1. [Node.js Setup](../04-integration/frameworks/nodejs.md)
2. [API Patterns](../03-patterns/api-patterns.md)
3. [Server Deployment](../04-integration/deployment.md#traditional-servers)

### Serverless Functions
**Read This**:
1. [Edge Runtime](../04-integration/edge-runtime.md)
2. [Serverless Patterns](../03-patterns/serverless-patterns.md)
3. [Cold Start Optimization](../03-patterns/performance-patterns.md#cold-starts)

### Docker/Containers
**Read This**:
1. [Container Setup](../04-integration/docker.md)
2. [Environment Configuration](../04-integration/environment.md)
3. [Scaling Patterns](../03-patterns/scaling-patterns.md)

## What Database/Storage?

### PostgreSQL
**Read This**:
- [PostgreSQL Setup](../04-integration/storage-setup.md#postgresql)
- [Memory with PostgreSQL](../02-api-reference/memory/index.md#postgresql-adapter)

### Redis/Upstash
**Read This**:
- [Redis Setup](../04-integration/storage-setup.md#redis)
- [Caching Patterns](../03-patterns/performance-patterns.md#caching)

### Vector Database
**For RAG/Semantic Search**:
- Pinecone → [Pinecone Setup](../04-integration/vector-stores.md#pinecone)
- ChromaDB → [ChromaDB Setup](../04-integration/vector-stores.md#chromadb)
- Qdrant → [Qdrant Setup](../04-integration/vector-stores.md#qdrant)

### No Database (Simple Start)
**Read This**:
- [Memory Storage](../02-api-reference/storage/index.md#memory-adapter)
- [Getting Started](../00-quick-start/minimal-example.md)

## Common Specific Scenarios

### "I want to build ChatGPT clone"
```
1. Chatbot Tutorial → Memory API → Streaming → Deployment
```

### "I want to analyze documents"
```
1. RAG Tutorial → Document Processing → Vector Search → Memory Integration
```

### "I want to automate tasks"
```
1. Workflow API → Tool Integration → Error Handling → Monitoring
```

### "I want to integrate with my API"
```
1. Custom Tools → API Integration → Error Handling → Authentication
```

### "I want to process user uploads"
```
1. File Processing → RAG API → Document Chunking → Vector Storage
```

### "I want multi-agent collaboration"
```
1. Multi-Agent Patterns → Workflow Coordination → Message Passing → State Management
```

## Decision Matrix

| If you need... | Core only | + Memory | + RAG | + Workflows |
|----------------|-----------|----------|-------|-------------|
| Simple Q&A | ✅ | | | |
| Chatbot | ✅ | ✅ | | |
| Knowledge base | ✅ | ✅ | ✅ | |
| Document analysis | ✅ | | ✅ | |
| Complex automation | ✅ | | | ✅ |
| Full AI assistant | ✅ | ✅ | ✅ | ✅ |

## Quick Navigation Commands

**For Agents**: Use these patterns to jump directly to relevant sections:

```typescript
// If building → Read tutorials
// If debugging → Read troubleshooting  
// If optimizing → Read performance
// If deploying → Read integration
// If learning → Read concepts
```

## Flow Charts

### Agent Creation Flow
```
Need AI response? → Agent API
Need structured data? → generateObject()
Need streaming? → stream()
Need tools? → Tool Integration
Need memory? → Memory API
```

### Problem Resolution Flow
```
Error occurred? → Troubleshooting
Performance issue? → Performance Patterns
Security concern? → Security Guide
Deployment issue? → Integration Guides
```

### Learning Flow
```
New to AI? → Prerequisites → Concepts → Examples
Experienced? → Architecture → API → Patterns
Specific goal? → Use this decision tree
```

## See Also

- [Search Index](./search-index.md) - Keyword-based lookup
- [Learning Path](./learning-path.md) - Structured reading order
- [API Method Index](./api-method-index.md) - Complete method reference