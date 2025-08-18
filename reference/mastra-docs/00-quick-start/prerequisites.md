# Prerequisites

> Required knowledge, tools, and setup before using Mastra

## Overview

This guide covers everything you need to know and have installed before starting with Mastra. Understanding these prerequisites ensures a smooth development experience.

## Required Knowledge

### JavaScript/TypeScript Fundamentals

- **ES6+ Features**: async/await, destructuring, modules
- **TypeScript Basics**: types, interfaces, generics
- **Node.js**: event loop, modules, npm/pnpm
- **Promises**: Promise chains, error handling

### AI/LLM Concepts

- **Language Models**: Understanding tokens, context windows
- **Prompting**: System prompts, few-shot examples
- **Tools/Functions**: Function calling, tool use
- **RAG**: Embeddings, vector search basics

## System Requirements

### Runtime Environment

```bash
# Check Node.js version (must be 20+)
node --version
# v20.11.0 or higher

# Check package manager
pnpm --version  # Recommended: 9.7.0+
npm --version   # Alternative: 10.0+
yarn --version  # Alternative: 1.22+
```

### Development Tools

```bash
# TypeScript (installed with Mastra)
npx tsc --version
# Version 5.0 or higher

# Git for version control
git --version
```

### Recommended IDE Setup

**VS Code Extensions**:
- TypeScript and JavaScript Language Features
- ESLint
- Prettier
- Environment Variables (.env support)

**VS Code Settings**:
```json
{
  "typescript.tsdk": "node_modules/typescript/lib",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}
```

## LLM Provider Requirements

### API Keys Required

At least one of:

1. **OpenAI**
   - Sign up: https://platform.openai.com
   - Get API key: https://platform.openai.com/api-keys
   - Models: GPT-4, GPT-3.5-turbo

2. **Anthropic**  
   - Sign up: https://console.anthropic.com
   - Get API key: Settings → API Keys
   - Models: Claude 3 Opus, Sonnet, Haiku

3. **Google Gemini**
   - Sign up: https://ai.google.dev
   - Get API key: Google AI Studio
   - Models: Gemini Pro, Gemini Flash

### Environment Setup

Create `.env` file:
```bash
# .env
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_GENERATIVE_AI_API_KEY=...
```

Load in application:
```typescript
import { config } from 'dotenv';
config(); // Load .env file
```

## Optional Dependencies

### Database (for persistence)

**PostgreSQL**:
```bash
# Local installation
brew install postgresql  # macOS
sudo apt-get install postgresql  # Ubuntu

# Or use Docker
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:15
```

**Redis (Upstash)**:
```bash
# Local Redis
brew install redis  # macOS
redis-server        # Start server

# Or use Upstash cloud
# Sign up at: https://upstash.com
```

### Vector Databases (for RAG)

**Pinecone**:
- Sign up: https://www.pinecone.io
- Get API key from dashboard
- Create index with dimension 1536 (OpenAI embeddings)

**ChromaDB**:
```bash
# Run with Docker
docker run -d \
  --name chroma \
  -p 8000:8000 \
  chromadb/chroma
```

### Docker (for local services)

```bash
# Install Docker Desktop
# https://www.docker.com/products/docker-desktop

# Verify installation
docker --version
docker compose version
```

## Project Structure Setup

### Basic Project Layout

```
my-mastra-app/
├── src/
│   ├── agents/       # Agent definitions
│   ├── workflows/    # Workflow definitions
│   ├── tools/        # Custom tools
│   └── index.ts      # Main entry point
├── .env              # Environment variables
├── .gitignore        # Git ignore file
├── package.json      # Dependencies
├── tsconfig.json     # TypeScript config
└── README.md         # Documentation
```

### TypeScript Configuration

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020"],
    "moduleResolution": "node",
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### Package.json Scripts

```json
{
  "scripts": {
    "dev": "mastra dev",
    "build": "tsc",
    "start": "node dist/index.js",
    "typecheck": "tsc --noEmit"
  }
}
```

## Verification Checklist

Run these commands to verify setup:

```bash
# 1. Node.js version
node --version  # Should show v20+

# 2. Package manager
pnpm --version  # Or npm/yarn

# 3. Create test project
npx create-mastra@latest test-app
cd test-app

# 4. Install dependencies
pnpm install

# 5. Set environment variable
echo "OPENAI_API_KEY=your-key" > .env

# 6. Run development server
pnpm dev
```

## Common Setup Issues

### Issue: Node.js version too old
```bash
# Install Node.js 20+ using nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

### Issue: pnpm not installed
```bash
# Install pnpm
npm install -g pnpm

# Or using corepack
corepack enable
corepack prepare pnpm@latest --activate
```

### Issue: TypeScript errors
```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
pnpm install
pnpm typecheck
```

## See Also

- [Installation](./installation.md)
- [Minimal Example](./minimal-example.md)
- [Development Environment](../04-integration/tooling.md)

## Next Steps

- [Install Mastra](./installation.md)
- [Run your first example](./minimal-example.md)
- [Understand core concepts](../01-core-concepts/architecture-overview.md)