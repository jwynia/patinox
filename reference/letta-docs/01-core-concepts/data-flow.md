# Data Flow

> How information moves through the Letta system

## Message Processing Flow

### High-Level Flow
```
User Message → Agent Processing → LLM Response → Tool Execution → Agent State Update
```

### Detailed Message Flow

```mermaid
graph TD
    A[User Input] --> B[MessageManager]
    B --> C[Agent.step()]
    C --> D[Memory Assembly]
    D --> E[LLM API Call]
    E --> F{Tool Calls?}
    F -->|Yes| G[Tool Execution]
    F -->|No| H[Response Processing]
    G --> I[Tool Results]
    I --> J[Update Context]
    J --> E
    H --> K[Memory Update]
    K --> L[State Persistence]
    L --> M[Response to User]
```

### Step-by-Step Breakdown

#### 1. Message Ingestion
```python
# User sends message
user_message = MessageCreate(role="user", content="What's my order status?")

# MessageManager processes input
message_manager.create_user_message(
    agent_id=agent_id,
    user=current_user,
    content=user_message.content
)
```

#### 2. Agent Processing
```python
# Agent receives message and processes it
agent = agent_manager.get_agent(agent_id, user=current_user)
response = await agent.step(user_message)

# Agent assembles context from memory
context = agent.assemble_context()
# Includes: core memory + recent messages + system prompt
```

#### 3. Memory Context Assembly
```python
# Core memory (always included)
core_memory = agent.memory.get_blocks()

# Recent message history (sliding window)
recent_messages = agent.memory.get_recent_messages(limit=50)

# System prompt and instructions
system_context = agent.get_system_prompt()

# Combined context sent to LLM
full_context = system_context + core_memory + recent_messages
```

#### 4. LLM Interaction
```python
# LLM client processes request
llm_response = await llm_client.chat_completion(
    messages=full_context,
    tools=agent.tools,
    model=agent.llm_config.model
)

# Response contains either text or tool calls
if llm_response.tool_calls:
    # Execute tools
    tool_results = await tool_executor.execute(llm_response.tool_calls)
else:
    # Process text response
    agent_response = llm_response.content
```

#### 5. Tool Execution Flow
```python
# For each tool call in LLM response
for tool_call in llm_response.tool_calls:
    # Validate tool exists and agent has access
    tool = tool_manager.get_tool(tool_call.name, user=current_user)
    
    # Execute in sandbox
    result = tool_executor.execute(
        tool=tool,
        arguments=tool_call.arguments,
        sandbox_config=agent.sandbox_config
    )
    
    # Package result for next LLM call
    tool_result = ToolResult(
        tool_call_id=tool_call.id,
        output=result.output,
        error=result.error
    )
```

#### 6. Memory Updates
```python
# Add messages to agent's memory
agent.memory.add_message(user_message)
agent.memory.add_message(assistant_message)

# Update core memory if needed
if memory_update_requested:
    agent.memory.update_block(
        label="human",
        value="Updated customer info..."
    )

# Add to archival memory for long-term storage
if important_information:
    agent.memory.add_to_archival(content)
```

#### 7. Context Window Management
```python
# Check if approaching context limit
if agent.approaching_context_limit():
    # Summarize older messages
    summary = summarizer.summarize_messages(
        messages=agent.memory.get_old_messages(),
        mode=agent.summarization_mode
    )
    
    # Replace old messages with summary
    agent.memory.replace_with_summary(summary)
    
    # Move details to archival memory
    agent.memory.archive_detailed_messages(old_messages)
```

## Memory Data Flow

### Memory Types and Their Flow

#### Core Memory Flow
```
Agent Creation → Initial Blocks → Runtime Updates → Persistence
```

```python
# Initial setup
agent = client.agents.create(
    memory_blocks=[
        CreateBlock(label="human", value="Customer: John Doe"),
        CreateBlock(label="persona", value="Helpful support agent")
    ]
)

# Runtime updates
agent.update_core_memory("human", "Customer: John Doe (VIP member)")
# Triggers: validation → update → persistence → embedding update
```

#### Message History Flow
```
New Message → Add to History → Context Window Check → Summarization (if needed)
```

```python
# Linear growth until limit
messages: [msg1, msg2, msg3, ..., msg_n]

# When limit reached:
# 1. Identify messages to summarize
old_messages = messages[0:split_point]
keep_messages = messages[split_point:]

# 2. Create summary
summary = summarize(old_messages)

# 3. Replace old messages
messages = [summary] + keep_messages
```

#### Archival Memory Flow
```
Information → Embedding Generation → Vector Storage → Search Index
```

```python
# Information added to archival
content = "Important customer preference: prefers email over phone"

# Generate embedding
embedding = embedding_client.create_embedding(content)

# Store with metadata
passage = Passage(
    content=content,
    embedding=embedding,
    metadata={"source": "conversation", "timestamp": now()}
)

# Index for search
archival_index.add(passage)
```

## Tool Execution Data Flow

### Tool Discovery and Execution

```
Tool Registration → Agent Assignment → Runtime Execution → Result Processing
```

#### 1. Tool Registration Flow
```python
# Function definition
def get_weather(location: str) -> str:
    """Get weather for a location."""
    return f"Sunny in {location}"

# Schema generation
tool_schema = generate_schema(get_weather)
# Automatic conversion to OpenAI tool format

# Registration
tool = tool_manager.register_tool(
    func=get_weather,
    user=current_user
)
```

#### 2. Agent Tool Assignment
```python
# Tools attached to agents
agent_tools = agent_manager.get_agent_tools(agent_id)

# Available during LLM calls
llm_response = llm_client.chat_completion(
    messages=context,
    tools=agent_tools  # Tools provided to LLM
)
```

#### 3. Execution Flow
```python
# LLM requests tool execution
tool_call = ToolCall(
    name="get_weather",
    arguments={"location": "San Francisco"}
)

# Sandbox execution
sandbox = tool_executor.get_sandbox(agent.sandbox_config)
result = sandbox.execute(
    tool_call.name,
    tool_call.arguments
)

# Error handling and result packaging
if result.success:
    tool_result = ToolResult(output=result.output)
else:
    tool_result = ToolResult(error=result.error)
```

## File and Source Data Flow

### File Processing Pipeline

```
File Upload → Content Extraction → Chunking → Embedding → Storage → Agent Access
```

#### 1. File Upload Flow
```python
# File uploaded by user
file = client.files.upload_file(
    file_path="document.pdf",
    description="Product manual"
)

# Triggers processing pipeline
processor = FileProcessor(file)
extracted_content = processor.extract_text()
```

#### 2. Content Processing Flow
```python
# Chunking for optimal embedding
chunks = chunker.chunk_text(
    text=extracted_content,
    chunk_size=1000,
    overlap=200
)

# Embedding generation
embeddings = []
for chunk in chunks:
    embedding = embedding_client.create_embedding(chunk.content)
    embeddings.append(embedding)

# Storage as passages
passages = []
for chunk, embedding in zip(chunks, embeddings):
    passage = Passage(
        content=chunk.content,
        embedding=embedding,
        source_id=source.id,
        metadata=chunk.metadata
    )
    passages.append(passage)
```

#### 3. Agent Access Flow
```python
# Agent searches across attached sources
query = "installation requirements"
results = agent.search_archival_memory(query)

# Semantic search across all passages
relevant_passages = vector_search(
    query_embedding=embed(query),
    passages=agent.accessible_passages,
    limit=10,
    threshold=0.7
)
```

## Error and Recovery Flow

### Error Propagation
```
Error Occurrence → Error Classification → Recovery Strategy → User Notification
```

#### Common Error Flows

##### LLM API Errors
```python
try:
    response = llm_client.chat_completion(messages)
except RateLimitError:
    # Exponential backoff retry
    await asyncio.sleep(backoff_time)
    response = llm_client.chat_completion(messages)
except ContextLengthError:
    # Force summarization
    agent.force_summarize()
    response = llm_client.chat_completion(shortened_messages)
```

##### Tool Execution Errors
```python
try:
    result = tool_executor.execute(tool_call)
except ToolTimeoutError:
    result = ToolResult(error="Tool execution timed out")
except ToolPermissionError:
    result = ToolResult(error="Tool access denied")
except Exception as e:
    result = ToolResult(error=f"Tool error: {str(e)}")

# Error results sent back to LLM for handling
```

##### Memory Errors
```python
try:
    agent.memory.update_block(label, value)
except MemoryBlockNotFound:
    # Create new block instead
    agent.memory.create_block(label, value)
except MemoryQuotaExceeded:
    # Trigger cleanup
    agent.memory.cleanup_old_passages()
```

## Performance and Optimization Flow

### Caching Strategies
```python
# LLM response caching
cache_key = hash(messages + model + tools)
if cached_response := cache.get(cache_key):
    return cached_response

response = llm_client.chat_completion(...)
cache.set(cache_key, response, ttl=3600)
```

### Database Optimization Flow
```python
# Connection pooling
with db_pool.get_connection() as conn:
    # Database operations

# Async batch operations
await asyncio.gather(
    save_messages(messages),
    update_embeddings(passages),
    log_metrics(usage_stats)
)
```

## Next Steps

- [Mental Model](mental-model.md) - The philosophy behind the design
- [Terminology](terminology.md) - Domain-specific terms
- [Architecture Overview](architecture-overview.md) - System organization

## See Also

- [Message Management](../03-patterns/common-use-cases/message-handling.md)
- [Tool Execution](../03-patterns/common-use-cases/tool-integration.md)
- [Memory Strategies](../03-patterns/memory-patterns.md)