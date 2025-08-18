# Import Patterns

> How to import and use Mastra modules in different contexts and build systems

## Overview

Mastra uses ES modules with TypeScript. This guide covers import patterns for different module systems, bundlers, and runtime environments.

## Basic Import Patterns

### Core Package Imports

```typescript
// Default import - Mastra class
import { Mastra } from '@mastra/core';

// Named imports - Common utilities
import { 
  Agent, 
  Workflow, 
  Step,
  createTool,
  RuntimeContext 
} from '@mastra/core';

// Type imports
import type { 
  MastraConfig,
  AgentConfig,
  WorkflowConfig,
  Tool 
} from '@mastra/core';
```

### Sub-module Imports

```typescript
// Agent module
import { Agent, MessageList } from '@mastra/core/agent';

// Workflow module  
import { Workflow, Step } from '@mastra/core/workflows';

// Tools module
import { createTool, validateTool } from '@mastra/core/tools';

// Memory module
import { MastraMemory } from '@mastra/core/memory';

// Storage module
import { Storage } from '@mastra/core/storage';
```

## Provider Imports

### AI SDK Providers

```typescript
// OpenAI
import { createOpenAI } from '@ai-sdk/openai';

// Anthropic
import { createAnthropic } from '@ai-sdk/anthropic';

// Google
import { createGoogleGenerativeAI } from '@ai-sdk/google';

// Multiple providers
import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';

const providers = {
  openai: createOpenAI({ apiKey: process.env.OPENAI_API_KEY }),
  anthropic: createAnthropic({ apiKey: process.env.ANTHROPIC_API_KEY }),
};
```

## Storage Adapter Imports

```typescript
// PostgreSQL
import { PgStorage } from '@mastra/pg';

// Upstash
import { UpstashStorage } from '@mastra/upstash';

// Pinecone
import { PineconeVector } from '@mastra/pinecone';

// ChromaDB
import { ChromaVector } from '@mastra/chroma';
```

## CommonJS Compatibility

### Using require (Node.js)

```javascript
// CommonJS require
const { Mastra } = require('@mastra/core');
const { createOpenAI } = require('@ai-sdk/openai');

// Async import for ESM-only packages
async function loadMastra() {
  const { Mastra } = await import('@mastra/core');
  return new Mastra({ /* config */ });
}
```

### TypeScript with CommonJS

```json
// tsconfig.json
{
  "compilerOptions": {
    "module": "commonjs",
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true
  }
}
```

## Build Tool Configurations

### Vite

```typescript
// vite.config.ts
export default {
  optimizeDeps: {
    include: ['@mastra/core', 'ai', 'zod'],
  },
  build: {
    target: 'es2020',
  },
};
```

### Next.js

```typescript
// next.config.js
module.exports = {
  transpilePackages: ['@mastra/core'],
  experimental: {
    serverComponentsExternalPackages: ['@mastra/core'],
  },
};
```

### Webpack

```javascript
// webpack.config.js
module.exports = {
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    alias: {
      '@mastra/core': path.resolve('node_modules/@mastra/core/dist'),
    },
  },
};
```

## Runtime-Specific Patterns

### Node.js

```typescript
import { Mastra } from '@mastra/core';
import { config } from 'dotenv';

config(); // Load environment variables

const mastra = new Mastra({ /* config */ });
```

### Edge Runtime (Vercel/Cloudflare)

```typescript
import { Mastra } from '@mastra/core';

// Use runtime config
const mastra = new Mastra({
  providers: {
    openai: createOpenAI({
      apiKey: process.env.OPENAI_API_KEY,
    }),
  },
});

export default {
  async fetch(request: Request) {
    const agent = mastra.getAgent('assistant');
    // Handle request
  },
};
```

### Browser (Client-side)

```typescript
// Use client SDK instead
import { MastraClient } from '@mastra/client-js';

const client = new MastraClient({
  url: 'https://api.example.com',
});
```

## Dynamic Imports

### Lazy Loading

```typescript
// Load on demand
async function loadAgent() {
  const { Agent } = await import('@mastra/core/agent');
  return new Agent({ /* config */ });
}

// Conditional loading
const storage = process.env.DATABASE_URL
  ? await import('@mastra/pg')
  : await import('@mastra/upstash');
```

### Code Splitting

```typescript
// Route-based splitting (Next.js)
const AgentChat = dynamic(() => import('@mastra/core/agent'), {
  loading: () => <p>Loading...</p>,
});
```

## Type-Only Imports

```typescript
// Import only types (removed at compile time)
import type { 
  MastraConfig,
  AgentConfig,
  Tool,
  WorkflowStep 
} from '@mastra/core';

// Separate type imports
import { Mastra } from '@mastra/core';
import type { MastraConfig } from '@mastra/core';

// Inline type imports
import { type MastraConfig, Mastra } from '@mastra/core';
```

## Module Resolution

### Package.json exports

```json
{
  "exports": {
    ".": "./dist/index.js",
    "./agent": "./dist/agent/index.js",
    "./workflows": "./dist/workflows/index.js",
    "./tools": "./dist/tools/index.js"
  }
}
```

### TypeScript paths

```json
// tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@mastra/*": ["./node_modules/@mastra/*/dist"]
    }
  }
}
```

## See Also

- [Installation](./installation.md)
- [Prerequisites](./prerequisites.md)
- [TypeScript Configuration](../04-integration/build-configuration.md)

## Next Steps

- [Configure your first agent](../01-core-concepts/key-abstractions.md)
- [Set up development environment](../04-integration/tooling.md)
- [Understand module structure](../06-advanced/internals/index.md)