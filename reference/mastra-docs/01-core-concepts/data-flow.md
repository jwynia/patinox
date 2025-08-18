# Data Flow

> How information moves through the Mastra system

## Overview

Understanding data flow in Mastra is crucial for building efficient AI applications. Data flows through multiple layers: from user input through agents and tools, to storage and memory systems, and back as responses.

## Agent Data Flow

### Basic Generation Flow

```
1. User Input
   ↓
2. Message Processing
   ├── Input validation
   ├── Context retrieval
   └── Message formatting
   ↓
3. LLM Provider Call
   ├── System prompt
   ├── User messages
   └── Tool definitions
   ↓
4. Response Processing
   ├── Tool execution (if needed)
   ├── Memory persistence
   └── Output formatting
   ↓
5. User Response
```

### Tool Execution Flow

```
LLM Decision
   ↓
Tool Selection
   ↓
Parameter Extraction
   ↓
Schema Validation
   ↓
Tool Executor
   ├── Integration call (if needed)
   ├── Database query (if needed)
   └── Computation
   ↓
Result Formatting
   ↓
Return to LLM
   ↓
Final Response
```

### Memory Integration Flow

```
New Message
   ↓
Thread Resolution
   ├── Find existing thread
   └── Create new thread
   ↓
Context Loading
   ├── Recent messages
   ├── Semantic recall
   └── Working memory
   ↓
LLM Processing
   ↓
Response Generation
   ↓
Memory Update
   ├── Save user message
   ├── Save assistant response
   └── Update embeddings
```

## Workflow Data Flow

### Linear Workflow

```
Trigger Data
   ↓
Step 1: Validate
   ├── Input schema check
   └── Business rules
   ↓
Step 2: Process
   ├── Transform data
   └── Enrich with external data
   ↓
Step 3: Store
   ├── Save to database
   └── Update cache
   ↓
Step 4: Notify
   ├── Send notifications
   └── Log completion
   ↓
Final Result
```

### Branching Workflow

```
Initial Data
   ↓
Decision Step
   ├─→ Condition A → Path A Steps → Merge
   ├─→ Condition B → Path B Steps → Merge
   └─→ Default → Default Steps → Merge
                                   ↓
                              Final Output
```

### Suspend/Resume Flow

```
Workflow Start
   ↓
Step 1 Complete
   ↓
Suspend Point
   ├── Save state
   ├── Return suspend data
   └── Wait for resume signal
   
[External Event]
   ↓
Resume Signal
   ├── Load state
   ├── Merge resume data
   └── Continue execution
   ↓
Remaining Steps
   ↓
Completion
```

## Storage Data Flow

### Write Path

```
Application Data
   ↓
Serialization
   ├── JSON encoding
   └── Compression (optional)
   ↓
Storage Adapter
   ├── Connection pooling
   ├── Transaction management
   └── Write operation
   ↓
Backend Storage
   ├── PostgreSQL
   ├── Redis
   └── DynamoDB
```

### Read Path

```
Query Request
   ↓
Cache Check
   ├─→ Cache Hit → Return Data
   └─→ Cache Miss ↓
                  Storage Adapter
                     ↓
                  Backend Query
                     ↓
                  Deserialization
                     ↓
                  Cache Update
                     ↓
                  Return Data
```

## RAG Data Flow

### Document Ingestion

```
Raw Document
   ↓
Text Extraction
   ├── PDF parsing
   ├── HTML cleaning
   └── Markdown processing
   ↓
Chunking
   ├── Sentence splitting
   ├── Overlap calculation
   └── Size limits
   ↓
Embedding Generation
   ├── Batch processing
   └── Model selection
   ↓
Vector Storage
   ├── Index creation
   └── Metadata attachment
```

### Query Flow

```
User Query
   ↓
Query Embedding
   ↓
Vector Search
   ├── Similarity calculation
   └── Top-K selection
   ↓
Reranking
   ├── Cross-encoder scoring
   └── Relevance filtering
   ↓
Context Assembly
   ├── Chunk ordering
   └── Token limit check
   ↓
LLM Generation
   ↓
Response
```

## Streaming Data Flow

### Text Streaming

```
Stream Request
   ↓
LLM Stream Init
   ↓
Token Generation Loop
   ├─→ Token → Client Chunk → Display
   ├─→ Token → Client Chunk → Display
   └─→ [End] → Stream Close → Final Processing
```

### Object Streaming

```
Schema Definition
   ↓
Stream Request
   ↓
Partial Object Loop
   ├─→ Field Update → Validation → Client Update
   ├─→ Field Update → Validation → Client Update
   └─→ Complete Object → Final Validation → Stream End
```

## Memory Data Flow

### Thread-Based Flow

```
User Message
   ↓
Thread ID Resolution
   ├── From context
   ├── From session
   └── Generate new
   ↓
Message Storage
   ├── User message
   ├── Timestamp
   └── Metadata
   ↓
Context Window
   ├── Last N messages
   ├── Token limit
   └── Relevance filter
   ↓
LLM Processing
   ↓
Response Storage
   ├── Assistant message
   ├── Tool calls
   └── Usage metrics
```

### Semantic Memory Flow

```
New Information
   ↓
Embedding Generation
   ↓
Vector Storage
   ├── Document ID
   ├── Embedding vector
   └── Metadata
   
Query Time:
Search Query
   ↓
Query Embedding
   ↓
Similarity Search
   ↓
Result Ranking
   ↓
Context Integration
```

## Integration Data Flow

### OAuth Flow

```
Integration Request
   ↓
Token Check
   ├─→ Valid → API Call
   └─→ Expired ↓
              Token Refresh
                 ↓
              New Token
                 ↓
              API Call
   ↓
Response Processing
   ├── Type validation
   └── Error handling
   ↓
Return to Application
```

### Webhook Flow

```
External Event
   ↓
Webhook Endpoint
   ↓
Signature Verification
   ↓
Payload Parsing
   ↓
Event Router
   ├─→ Handler A
   ├─→ Handler B
   └─→ Handler C
   ↓
Processing
   ↓
Response
```

## Error Flow

### Retry Flow

```
Operation Attempt
   ↓
Error Occurs
   ↓
Retry Decision
   ├── Check retry count
   ├── Check error type
   └── Check backoff time
   ↓
[Retry] → Backoff Wait → Operation Attempt
[No Retry] → Error Propagation → Error Handler
```

## Performance Optimizations

### Caching Layers

```
Request
   ↓
L1 Cache (Memory)
   ├─→ Hit → Return
   └─→ Miss ↓
           L2 Cache (Redis)
              ├─→ Hit → Update L1 → Return
              └─→ Miss ↓
                      Database
                         ↓
                      Update L2
                         ↓
                      Update L1
                         ↓
                      Return
```

## See Also

- [Architecture Overview](./architecture-overview.md)
- [Key Abstractions](./key-abstractions.md)
- [Mental Model](./mental-model.md)

## Next Steps

- [Understand the mental model](./mental-model.md)
- [Learn about streaming](../03-patterns/async-patterns.md)
- [Explore caching strategies](../03-patterns/performance-patterns.md)