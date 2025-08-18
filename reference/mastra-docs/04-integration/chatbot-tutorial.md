# Chatbot Tutorial

> Complete guide to building a conversational AI with memory and RAG

## Overview

This tutorial walks through building a production-ready chatbot using Mastra. You'll create an agent with persistent memory, integrate with a knowledge base using RAG, and deploy to production with proper error handling and monitoring.

## What You'll Build

- **Conversational agent** with personality and context
- **Memory system** for persistent conversations
- **Knowledge base** powered by RAG
- **File upload** for document ingestion
- **Streaming responses** for real-time chat
- **Error handling** and recovery
- **Production deployment** ready

## Prerequisites

- Node.js 20+ installed
- OpenAI API key (or other LLM provider)
- PostgreSQL database (local or cloud)
- Basic TypeScript knowledge

## Step 1: Project Setup

### Initialize Project

```bash
npx create-mastra@latest my-chatbot
cd my-chatbot
```

### Install Additional Dependencies

```bash
pnpm add @mastra/memory @mastra/rag @mastra/pg
pnpm add @pinecone-database/pinecone multer express cors
pnpm add -D @types/multer @types/express @types/cors
```

### Environment Configuration

```bash
# .env
OPENAI_API_KEY=sk-your-openai-key
DATABASE_URL=postgresql://user:password@localhost:5432/chatbot
PINECONE_API_KEY=your-pinecone-key
PINECONE_INDEX=chatbot-knowledge
```

## Step 2: Core Configuration

### Setup Mastra Instance

```typescript
// src/mastra/index.ts
import { Mastra } from '@mastra/core';
import { MastraMemory } from '@mastra/memory';
import { RAG } from '@mastra/rag';
import { PgStorage } from '@mastra/pg';
import { PineconeVector } from '@mastra/pinecone';
import { createOpenAI } from '@ai-sdk/openai';

// Storage setup
const storage = new PgStorage({
  connectionString: process.env.DATABASE_URL!
});

// Vector store for RAG
const vectorStore = new PineconeVector({
  apiKey: process.env.PINECONE_API_KEY!,
  index: process.env.PINECONE_INDEX!
});

// Memory system
const memory = new MastraMemory({
  name: 'chatbot-memory',
  storage,
  vectorStore,
  embedding: {
    provider: 'openai',
    model: 'text-embedding-3-small'
  },
  maxMessages: 100,
  maxTokens: 8000
});

// RAG system
const rag = new RAG({
  name: 'chatbot-knowledge',
  vectorStore,
  embedding: {
    provider: 'openai',
    model: 'text-embedding-3-small'
  },
  chunking: {
    strategy: 'sentence',
    size: 512,
    overlap: 50
  }
});

// Main Mastra instance
export const mastra = new Mastra({
  providers: {
    openai: createOpenAI({
      apiKey: process.env.OPENAI_API_KEY
    })
  },
  agents: {
    chatbot: {
      model: 'openai:gpt-4o',
      instructions: `You are a helpful AI assistant. You have access to a knowledge base and conversation history.
      
      Guidelines:
      - Be friendly, professional, and helpful
      - Use the conversation history to maintain context
      - When relevant information is available in the knowledge base, reference it
      - If you don't know something, say so honestly
      - Ask clarifying questions when needed
      - Keep responses concise but informative`,
      
      memory: {
        type: 'thread',
        instance: memory
      },
      
      tools: {
        searchKnowledge: {
          description: 'Search the knowledge base for relevant information',
          schema: z.object({
            query: z.string().describe('Search query for the knowledge base')
          }),
          executor: async ({ query }) => {
            const results = await rag.query(query, { limit: 5 });
            return {
              results: results.map(r => ({
                content: r.content,
                score: r.score,
                source: r.metadata?.source
              }))
            };
          }
        },
        
        getConversationContext: {
          description: 'Get relevant context from conversation history',
          schema: z.object({
            query: z.string().describe('What to search for in conversation history'),
            threadId: z.string().describe('Current conversation thread ID')
          }),
          executor: async ({ query, threadId }) => {
            const context = await memory.recall({
              threadId,
              context: query,
              limit: 5
            });
            return {
              relevantMessages: context.messages,
              summary: context.summary
            };
          }
        }
      }
    }
  },
  storage,
  memory,
  rag
});
```

## Step 3: API Routes

### Chat Endpoint

```typescript
// src/routes/chat.ts
import express from 'express';
import { mastra } from '../mastra';
import { z } from 'zod';

const router = express.Router();

const chatSchema = z.object({
  message: z.string().min(1).max(1000),
  threadId: z.string().optional(),
  stream: z.boolean().optional().default(false)
});

// Stream chat endpoint
router.post('/stream', async (req, res) => {
  try {
    const { message, threadId = `thread-${Date.now()}`, stream } = chatSchema.parse(req.body);
    
    const agent = mastra.getAgent('chatbot');
    
    if (stream) {
      // Set up SSE
      res.writeHead(200, {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive',
        'Access-Control-Allow-Origin': '*'
      });
      
      const streamResult = await agent.stream(message, { threadId });
      
      for await (const chunk of streamResult) {
        res.write(`data: ${JSON.stringify({ type: 'chunk', content: chunk })}\n\n`);
      }
      
      res.write(`data: ${JSON.stringify({ type: 'done' })}\n\n`);
      res.end();
    } else {
      // Regular response
      const result = await agent.generate(message, { threadId });
      
      res.json({
        response: result.text,
        threadId,
        usage: result.usage,
        toolCalls: result.toolCalls
      });
    }
  } catch (error) {
    console.error('Chat error:', error);
    res.status(500).json({ 
      error: error.message,
      type: 'chat_error'
    });
  }
});

// Get conversation history
router.get('/history/:threadId', async (req, res) => {
  try {
    const { threadId } = req.params;
    const { limit = 50 } = req.query;
    
    const messages = await mastra.memory.getMessages({
      threadId,
      limit: Number(limit)
    });
    
    res.json({ messages });
  } catch (error) {
    console.error('History error:', error);
    res.status(500).json({ error: error.message });
  }
});

export default router;
```

### Document Upload

```typescript
// src/routes/documents.ts
import express from 'express';
import multer from 'multer';
import { mastra } from '../mastra';
import { extractTextFromFile } from '../utils/fileProcessor';

const router = express.Router();
const upload = multer({ 
  dest: 'uploads/',
  limits: { fileSize: 10 * 1024 * 1024 } // 10MB limit
});

router.post('/upload', upload.single('document'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }
    
    // Extract text from uploaded file
    const text = await extractTextFromFile(req.file.path, req.file.mimetype);
    
    // Add to knowledge base
    const documentId = await mastra.rag.add({
      content: text,
      metadata: {
        filename: req.file.originalname,
        uploadedAt: new Date().toISOString(),
        size: req.file.size,
        type: req.file.mimetype
      }
    });
    
    // Clean up uploaded file
    await fs.unlink(req.file.path);
    
    res.json({
      success: true,
      documentId,
      filename: req.file.originalname,
      size: req.file.size
    });
    
  } catch (error) {
    console.error('Upload error:', error);
    res.status(500).json({ error: error.message });
  }
});

// List documents
router.get('/list', async (req, res) => {
  try {
    const documents = await mastra.rag.listDocuments();
    res.json({ documents });
  } catch (error) {
    console.error('List error:', error);
    res.status(500).json({ error: error.message });
  }
});

// Delete document
router.delete('/:documentId', async (req, res) => {
  try {
    const { documentId } = req.params;
    await mastra.rag.delete(documentId);
    res.json({ success: true });
  } catch (error) {
    console.error('Delete error:', error);
    res.status(500).json({ error: error.message });
  }
});

export default router;
```

## Step 4: File Processing Utilities

```typescript
// src/utils/fileProcessor.ts
import fs from 'fs/promises';
import pdf from 'pdf-parse';

export async function extractTextFromFile(filePath: string, mimeType: string): Promise<string> {
  switch (mimeType) {
    case 'text/plain':
      return await fs.readFile(filePath, 'utf-8');
      
    case 'application/pdf':
      const dataBuffer = await fs.readFile(filePath);
      const pdfData = await pdf(dataBuffer);
      return pdfData.text;
      
    case 'text/markdown':
      const mdContent = await fs.readFile(filePath, 'utf-8');
      // Remove markdown formatting for cleaner text
      return mdContent.replace(/[#*`\[\]]/g, '');
      
    default:
      throw new Error(`Unsupported file type: ${mimeType}`);
  }
}
```

## Step 5: Frontend Integration

### React Chat Component

```typescript
// src/components/ChatInterface.tsx
import React, { useState, useEffect, useRef } from 'react';

interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export function ChatInterface() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [threadId, setThreadId] = useState(`thread-${Date.now()}`);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const sendMessage = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: input,
      timestamp: new Date()
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      // Use streaming for real-time responses
      const response = await fetch('/api/chat/stream', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: input,
          threadId,
          stream: true
        })
      });

      const reader = response.body?.getReader();
      if (!reader) throw new Error('No response stream');

      let assistantContent = '';
      const assistantMessage: Message = {
        id: `msg-${Date.now() + 1}`,
        role: 'assistant',
        content: '',
        timestamp: new Date()
      };

      setMessages(prev => [...prev, assistantMessage]);

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = new TextDecoder().decode(value);
        const lines = chunk.split('\n');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const data = JSON.parse(line.slice(6));
              if (data.type === 'chunk') {
                assistantContent += data.content;
                setMessages(prev => prev.map(msg =>
                  msg.id === assistantMessage.id
                    ? { ...msg, content: assistantContent }
                    : msg
                ));
              }
            } catch (e) {
              // Ignore parsing errors
            }
          }
        }
      }
    } catch (error) {
      console.error('Chat error:', error);
      setMessages(prev => [...prev, {
        id: `error-${Date.now()}`,
        role: 'assistant',
        content: 'Sorry, I encountered an error. Please try again.',
        timestamp: new Date()
      }]);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="chat-container">
      <div className="messages">
        {messages.map(message => (
          <div key={message.id} className={`message ${message.role}`}>
            <div className="content">{message.content}</div>
            <div className="timestamp">
              {message.timestamp.toLocaleTimeString()}
            </div>
          </div>
        ))}
        {isLoading && (
          <div className="message assistant loading">
            <div className="typing-indicator">●●●</div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      
      <div className="input-area">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Type your message..."
          disabled={isLoading}
        />
        <button onClick={sendMessage} disabled={isLoading || !input.trim()}>
          Send
        </button>
      </div>
    </div>
  );
}
```

## Step 6: Error Handling & Recovery

```typescript
// src/middleware/errorHandler.ts
import { Request, Response, NextFunction } from 'express';

export class ChatbotError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500
  ) {
    super(message);
    this.name = 'ChatbotError';
  }
}

export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
) {
  console.error('Error:', error);

  if (error instanceof ChatbotError) {
    return res.status(error.statusCode).json({
      error: error.message,
      code: error.code
    });
  }

  // LLM provider errors
  if (error.message.includes('rate_limit')) {
    return res.status(429).json({
      error: 'Rate limit exceeded. Please try again in a moment.',
      code: 'RATE_LIMIT',
      retryAfter: 60
    });
  }

  if (error.message.includes('context_length')) {
    return res.status(400).json({
      error: 'Message too long. Please try a shorter message.',
      code: 'CONTEXT_TOO_LONG'
    });
  }

  // Database errors
  if (error.message.includes('connection')) {
    return res.status(503).json({
      error: 'Service temporarily unavailable. Please try again.',
      code: 'SERVICE_UNAVAILABLE'
    });
  }

  // Default error
  res.status(500).json({
    error: 'An unexpected error occurred.',
    code: 'INTERNAL_ERROR'
  });
}

// Graceful shutdown
process.on('SIGTERM', async () => {
  console.log('Shutting down gracefully...');
  // Close database connections, etc.
  await mastra.storage.close();
  process.exit(0);
});
```

## Step 7: Production Deployment

### Docker Configuration

```dockerfile
# Dockerfile
FROM node:20-alpine

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci --only=production

# Copy source
COPY . .

# Build application
RUN npm run build

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Start application
CMD ["npm", "start"]
```

### Environment Configuration

```yaml
# docker-compose.yml
version: '3.8'
services:
  chatbot:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgresql://postgres:password@db:5432/chatbot
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=chatbot
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
```

### Monitoring Setup

```typescript
// src/middleware/monitoring.ts
import { Request, Response, NextFunction } from 'express';

// Request logging
export function requestLogger(req: Request, res: Response, next: NextFunction) {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`${req.method} ${req.path} ${res.statusCode} ${duration}ms`);
    
    // Send metrics to monitoring service
    if (process.env.NODE_ENV === 'production') {
      sendMetrics({
        method: req.method,
        path: req.path,
        statusCode: res.statusCode,
        duration,
        timestamp: new Date()
      });
    }
  });
  
  next();
}

// Health check endpoint
export function healthCheck(req: Request, res: Response) {
  const health = {
    status: 'healthy',
    timestamp: new Date(),
    uptime: process.uptime(),
    memory: process.memoryUsage(),
    version: process.env.npm_package_version
  };
  
  res.json(health);
}
```

## Step 8: Testing

```typescript
// tests/chatbot.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { mastra } from '../src/mastra';

describe('Chatbot Integration', () => {
  beforeEach(async () => {
    // Setup test environment
    await mastra.storage.init();
  });

  it('should respond to basic questions', async () => {
    const agent = mastra.getAgent('chatbot');
    const result = await agent.generate('Hello, how are you?');
    
    expect(result.text).toBeDefined();
    expect(result.text.length).toBeGreaterThan(0);
  });

  it('should maintain conversation context', async () => {
    const agent = mastra.getAgent('chatbot');
    const threadId = 'test-thread-123';
    
    // First message
    await agent.generate('My name is John', { threadId });
    
    // Second message should remember the name
    const result = await agent.generate('What is my name?', { threadId });
    
    expect(result.text.toLowerCase()).toContain('john');
  });

  it('should search knowledge base', async () => {
    // Add test document
    await mastra.rag.add({
      content: 'The company was founded in 2020 by Alice Smith.',
      metadata: { source: 'company-info' }
    });
    
    const agent = mastra.getAgent('chatbot');
    const result = await agent.generate('When was the company founded?');
    
    expect(result.text).toContain('2020');
  });
});
```

## Step 9: Performance Optimization

```typescript
// src/utils/optimization.ts

// Response caching
const responseCache = new Map();

export function cacheResponse(key: string, response: any, ttl: number = 300000) {
  responseCache.set(key, {
    data: response,
    expires: Date.now() + ttl
  });
}

export function getCachedResponse(key: string) {
  const cached = responseCache.get(key);
  if (cached && cached.expires > Date.now()) {
    return cached.data;
  }
  responseCache.delete(key);
  return null;
}

// Context window management
export async function optimizeContext(messages: any[], maxTokens: number = 4000) {
  // Keep recent messages and most relevant historical context
  const recentMessages = messages.slice(-10);
  const tokenCount = estimateTokens(recentMessages);
  
  if (tokenCount <= maxTokens) {
    return recentMessages;
  }
  
  // Use memory to find most relevant older messages
  const relevantContext = await mastra.memory.recall({
    threadId: messages[0]?.threadId,
    context: messages[messages.length - 1]?.content,
    limit: 5
  });
  
  return [...relevantContext.messages, ...recentMessages.slice(-5)];
}
```

## Next Steps

1. **Scale the infrastructure** - Add load balancing, Redis caching
2. **Enhance the knowledge base** - Add more document types, better chunking
3. **Improve responses** - Fine-tune prompts, add response templates
4. **Add authentication** - User management, conversation isolation
5. **Monitor performance** - Add proper observability, alerting

## Troubleshooting

### Common Issues

**Memory not persisting**:
- Check database connection
- Verify thread IDs are consistent
- Ensure storage is properly initialized

**RAG not finding relevant documents**:
- Check embedding model consistency
- Verify document chunking strategy
- Test similarity thresholds

**Slow responses**:
- Implement response caching
- Optimize context window size
- Use faster embedding models

## See Also

- [Agent API](../02-api-reference/agents/index.md)
- [Memory API](../02-api-reference/memory/index.md)
- [RAG API](../02-api-reference/rag/index.md)
- [Performance Patterns](../03-patterns/performance-patterns.md)