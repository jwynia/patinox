# Installation

> Complete guide for installing Mastra and its dependencies in your project

## Overview

Mastra is a TypeScript framework for building AI applications with agents, workflows, RAG, and integrations. It requires Node.js v20+ and uses pnpm as the preferred package manager, though npm and yarn are also supported.

## System Requirements

- **Node.js**: v20.0 or higher
- **Package Manager**: pnpm (v9.7.0+), npm, or yarn
- **TypeScript**: v5.0 or higher (included as dependency)
- **Operating System**: macOS, Linux, Windows (WSL recommended)

## Installation Methods

### Using create-mastra (Recommended)

The fastest way to start a new Mastra project:

```bash
npx create-mastra@latest my-app
cd my-app
npm run dev
```

This scaffolds a complete project with:
- Pre-configured TypeScript setup
- Basic agent and workflow examples
- Development server with playground UI
- Environment variable template

### Manual Installation

Install core package in existing project:

```bash
# Using pnpm (recommended)
pnpm add @mastra/core

# Using npm
npm install @mastra/core

# Using yarn
yarn add @mastra/core
```

### Additional Packages

Install based on your needs:

```bash
# CLI and development tools
pnpm add -D @mastra/cli

# Memory systems
pnpm add @mastra/memory

# RAG (Retrieval-Augmented Generation)
pnpm add @mastra/rag

# Evaluation framework
pnpm add @mastra/evals

# Model Context Protocol
pnpm add @mastra/mcp
```

## Storage Adapters

Choose storage backend for persistence:

```bash
# PostgreSQL
pnpm add @mastra/pg

# Upstash Redis
pnpm add @mastra/upstash

# Pinecone vector database
pnpm add @mastra/pinecone

# ChromaDB
pnpm add @mastra/chroma
```

## LLM Provider Setup

Mastra uses Vercel AI SDK. Install providers:

```bash
# OpenAI
pnpm add @ai-sdk/openai

# Anthropic
pnpm add @ai-sdk/anthropic

# Google
pnpm add @ai-sdk/google

# Multiple providers
pnpm add @ai-sdk/openai @ai-sdk/anthropic
```

## Environment Variables

Create `.env` file in project root:

```env
# LLM Provider Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_GENERATIVE_AI_API_KEY=...

# Optional: Database URLs
DATABASE_URL=postgresql://...
REDIS_URL=redis://...
```

## Verify Installation

Check installation:

```bash
# Check Node version
node --version  # Should be v20+

# Check package installation
npm list @mastra/core

# Run development server (if using create-mastra)
npm run dev
```

## Docker Services (Optional)

For local development with databases:

```bash
# Start services
pnpm dev:services:up

# Stop services
pnpm dev:services:down
```

## Common Issues

### Memory Errors During Build
```bash
NODE_OPTIONS="--max-old-space-size=4096" npm run build
```

### Missing Peer Dependencies
```bash
npm install zod ai @ai-sdk/openai
```

## See Also

- [Minimal Example](./minimal-example.md)
- [Import Patterns](./import-patterns.md)
- [Prerequisites](./prerequisites.md)

## Next Steps

- [Create your first agent](../01-core-concepts/key-abstractions.md)
- [Build a workflow](../02-api-reference/workflows/index.md)
- [Set up integrations](../04-integration/frameworks/index.md)