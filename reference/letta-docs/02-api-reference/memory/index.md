# Memory API

> Manage agent memory blocks and archival storage

## Overview

The Memory API provides fine-grained control over agent memory systems. Letta agents have multi-layered memory:

- **Core Memory**: Essential blocks always in context (persona, human info, current context)
- **Archival Memory**: Searchable long-term storage using embeddings
- **Message History**: Conversation flow with automatic summarization

## Quick Start

```python
from letta_client import Letta

client = Letta(base_url="http://localhost:8283")

# Get agent memory
memory = client.agents.memory.retrieve(agent_id="agent_123")

# Update memory block
client.agents.memory.update_block(
    agent_id="agent_123",
    block_id="human_block_id",
    value="Customer: John Doe - VIP member since 2020"
)

# Search archival memory
results = client.agents.memory.archival.search(
    agent_id="agent_123",
    query="customer preferences",
    limit=5
)
```

## Core Memory Operations

### Retrieve Memory

Get complete memory state for an agent.

```python
def memory.retrieve(
    self,
    agent_id: str
) -> Memory:
    """Retrieve agent's complete memory state.
    
    Args:
        agent_id: UUID of the target agent
        
    Returns:
        Memory: Complete memory object with blocks, messages, and metadata
        
    Raises:
        ApiError: If agent not found or access denied
    """
```

#### Parameters

| Parameter | Type | Required | Constraints | Description |
|-----------|------|----------|-------------|-------------|
| `agent_id` | str | **Yes** | Valid UUID | Agent identifier |

#### Response Schema

```python
class Memory:
    blocks: List[Block]             # Core memory blocks
    messages: List[Message]         # Recent conversation history
    archival_memory: ArchivalMemory # Searchable long-term storage
    
class Block:
    id: str                         # Block UUID
    label: str                      # Block identifier (human, persona, etc.)
    value: str                      # Block content
    created_at: datetime           # Creation timestamp
    updated_at: datetime           # Last modification
```

**Example:**
```python
memory = client.agents.memory.retrieve(agent_id="agent_123")
print(f"Core blocks: {len(memory.blocks)}")
print(f"Recent messages: {len(memory.messages)}")

for block in memory.blocks:
    print(f"{block.label}: {block.value[:50]}...")
```

### Update Memory Block

Modify the content of a specific memory block.

```python
def memory.update_block(
    self,
    agent_id: str,
    block_id: str,
    value: str
) -> Block:
    """Update the content of a memory block.
    
    Args:
        agent_id: UUID of the target agent
        block_id: UUID of the memory block to update
        value: New content for the block
        
    Returns:
        Block: Updated memory block
        
    Raises:
        ApiError: If agent/block not found
        ValidationError: If value exceeds limits
    """
```

#### Parameters

| Parameter | Type | Required | Constraints | Description |
|-----------|------|----------|-------------|-------------|
| `agent_id` | str | **Yes** | Valid UUID | Agent identifier |
| `block_id` | str | **Yes** | Valid UUID | Memory block identifier |
| `value` | str | **Yes** | 1-2048 chars | New block content |

**Example:**
```python
updated_block = client.agents.memory.update_block(
    agent_id="agent_123",
    block_id="human_block_id",
    value="Customer: John Doe - VIP member, Account Manager: Sarah"
)

print(f"Updated {updated_block.label}: {updated_block.value}")
```

### Create Memory Block

Add a new memory block to an agent.

```python
def memory.create_block(
    self,
    agent_id: str,
    label: str,
    value: str
) -> Block:
    """Create a new memory block for an agent.
    
    Args:
        agent_id: UUID of the target agent
        label: Identifier for the new block
        value: Content for the block
        
    Returns:
        Block: Created memory block
        
    Raises:
        ApiError: If agent not found or label conflicts
        ValidationError: If parameters invalid
    """
```

#### Parameters

| Parameter | Type | Required | Constraints | Description |
|-----------|------|----------|-------------|-------------|
| `agent_id` | str | **Yes** | Valid UUID | Agent identifier |
| `label` | str | **Yes** | 1-32 chars, alphanumeric + underscores | Block identifier |
| `value` | str | **Yes** | 1-2048 chars | Block content |

**Example:**
```python
new_block = client.agents.memory.create_block(
    agent_id="agent_123",
    label="preferences",
    value="Prefers email communication, morning appointments"
)

print(f"Created block: {new_block.label}")
```

### Delete Memory Block

Remove a memory block from an agent.

```python
def memory.delete_block(
    self,
    agent_id: str,
    block_id: str
) -> None:
    """Delete a memory block from an agent.
    
    Args:
        agent_id: UUID of the target agent
        block_id: UUID of the memory block to delete
        
    Raises:
        ApiError: If agent/block not found
        ValidationError: If trying to delete required blocks
    """
```

#### Parameters

| Parameter | Type | Required | Constraints | Description |
|-----------|------|----------|-------------|-------------|
| `agent_id` | str | **Yes** | Valid UUID | Agent identifier |
| `block_id` | str | **Yes** | Valid UUID | Memory block identifier |

**Note**: Cannot delete required blocks (human, persona) that are essential for agent operation.

## Archival Memory Operations

### Search Archival Memory

Find relevant information in long-term storage using semantic search.

```python
def memory.archival.search(
    self,
    agent_id: str,
    query: str,
    limit: int = 10,
    threshold: float = 0.7
) -> List[Passage]:
    """Search archival memory using semantic similarity.
    
    Args:
        agent_id: UUID of the target agent
        query: Search query text
        limit: Maximum number of results to return
        threshold: Minimum similarity threshold (0.0-1.0)
        
    Returns:
        List[Passage]: Relevant passages ordered by similarity
        
    Raises:
        ApiError: If agent not found
        ValidationError: If parameters invalid
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `agent_id` | str | **Yes** | N/A | Valid UUID | Agent identifier |
| `query` | str | **Yes** | N/A | 1-512 chars | Search query |
| `limit` | int | No | 10 | 1-100 | Max results |
| `threshold` | float | No | 0.7 | 0.0-1.0 | Similarity threshold |

#### Response Schema

```python
class Passage:
    id: str                         # Passage UUID
    content: str                    # Passage text content
    similarity_score: float         # Similarity to query (0.0-1.0)
    metadata: dict                  # Source info, timestamps, etc.
    created_at: datetime           # When passage was created
```

**Example:**
```python
results = client.agents.memory.archival.search(
    agent_id="agent_123",
    query="customer payment history",
    limit=5,
    threshold=0.8
)

for passage in results:
    print(f"Score: {passage.similarity_score:.2f}")
    print(f"Content: {passage.content[:100]}...")
    print("---")
```

### Insert into Archival Memory

Add new information to long-term searchable storage.

```python
def memory.archival.insert(
    self,
    agent_id: str,
    content: str,
    metadata: Optional[dict] = None
) -> Passage:
    """Insert new content into archival memory.
    
    Args:
        agent_id: UUID of the target agent
        content: Text content to store
        metadata: Optional metadata for the passage
        
    Returns:
        Passage: Created passage with generated embedding
        
    Raises:
        ApiError: If agent not found
        ValidationError: If content invalid
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `agent_id` | str | **Yes** | N/A | Valid UUID | Agent identifier |
| `content` | str | **Yes** | N/A | 1-8192 chars | Content to store |
| `metadata` | dict | No | {} | Max 1KB JSON | Additional info |

**Example:**
```python
passage = client.agents.memory.archival.insert(
    agent_id="agent_123",
    content="Customer John Doe prefers email notifications and has a history of prompt payments.",
    metadata={
        "source": "customer_interaction",
        "category": "preferences",
        "importance": "high"
    }
)

print(f"Stored passage: {passage.id}")
```

### Delete from Archival Memory

Remove specific passages from long-term storage.

```python
def memory.archival.delete(
    self,
    agent_id: str,
    passage_id: str
) -> None:
    """Delete a passage from archival memory.
    
    Args:
        agent_id: UUID of the target agent
        passage_id: UUID of the passage to delete
        
    Raises:
        ApiError: If agent/passage not found
    """
```

## Message History Operations

### List Messages

Get conversation history for an agent.

```python
def memory.messages.list(
    self,
    agent_id: str,
    limit: int = 100,
    cursor: Optional[str] = None,
    start_date: Optional[str] = None,
    end_date: Optional[str] = None
) -> List[Message]:
    """List conversation messages for an agent.
    
    Args:
        agent_id: UUID of the target agent
        limit: Maximum number of messages
        cursor: Pagination cursor
        start_date: Filter start date (ISO format)
        end_date: Filter end date (ISO format)
        
    Returns:
        List[Message]: Messages ordered by timestamp (newest first)
        
    Raises:
        ApiError: If agent not found
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `agent_id` | str | **Yes** | N/A | Valid UUID | Agent identifier |
| `limit` | int | No | 100 | 1-1000 | Max messages |
| `cursor` | str | No | None | Valid cursor | Pagination token |
| `start_date` | str | No | None | ISO 8601 | Filter start |
| `end_date` | str | No | None | ISO 8601 | Filter end |

**Example:**
```python
# Get recent messages
recent = client.agents.memory.messages.list(
    agent_id="agent_123",
    limit=20
)

# Get messages from specific date range
filtered = client.agents.memory.messages.list(
    agent_id="agent_123",
    start_date="2024-01-01T00:00:00Z",
    end_date="2024-01-31T23:59:59Z"
)

for message in recent:
    print(f"{message.role}: {message.content[:50]}...")
```

## Memory Management Strategies

### Memory Block Best Practices

```python
# Good: Focused, specific blocks
memory_blocks = [
    CreateBlock(label="human", value="Customer: John Doe - VIP since 2020"),
    CreateBlock(label="persona", value="Professional customer service agent"),
    CreateBlock(label="context", value="Handling billing inquiry for order #12345"),
    CreateBlock(label="priorities", value="1. Resolve issue quickly 2. Maintain satisfaction")
]

# Bad: Overloaded single block
memory_blocks = [
    CreateBlock(
        label="everything", 
        value="Customer John VIP since 2020 billing issue order 12345 professional agent resolve quickly maintain satisfaction..."
    )
]
```

### Archival Memory Optimization

```python
# Structured information storage
def store_customer_interaction(agent_id: str, customer_info: dict):
    content = f"""
    Customer: {customer_info['name']}
    Issue: {customer_info['issue']}
    Resolution: {customer_info['resolution']}
    Satisfaction: {customer_info['rating']}/5
    Follow-up needed: {customer_info['follow_up']}
    """
    
    client.agents.memory.archival.insert(
        agent_id=agent_id,
        content=content,
        metadata={
            "customer_id": customer_info['id'],
            "interaction_type": "support",
            "resolved": customer_info['resolved'],
            "timestamp": datetime.now().isoformat()
        }
    )
```

## Error Handling

### Common Memory Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Block not found | Invalid block_id | Use `memory.retrieve()` to get valid IDs |
| Label conflict | Duplicate label name | Choose unique label or update existing |
| Content too large | Block/passage exceeds limits | Split content or summarize |
| Required block deletion | Trying to delete human/persona | Update instead of delete |

```python
try:
    block = client.agents.memory.update_block(
        agent_id="agent_123",
        block_id="invalid_id",
        value="New content"
    )
except ApiError as e:
    if e.status_code == 404:
        # Get valid block IDs
        memory = client.agents.memory.retrieve("agent_123")
        valid_ids = [block.id for block in memory.blocks]
        print(f"Valid block IDs: {valid_ids}")
    elif e.status_code == 422:
        if "content too large" in e.message.lower():
            print("Content exceeds 2048 character limit")
```

## Performance Considerations

### Memory Size Management

```python
def check_memory_health(agent_id: str):
    """Check and optimize agent memory usage."""
    memory = client.agents.memory.retrieve(agent_id)
    
    # Check core memory size
    total_core_size = sum(len(block.value) for block in memory.blocks)
    if total_core_size > 4000:
        print(f"Core memory large: {total_core_size} chars")
    
    # Check message history
    if len(memory.messages) > 500:
        print(f"Large message history: {len(memory.messages)} messages")
    
    # Check for very large blocks
    for block in memory.blocks:
        if len(block.value) > 1000:
            print(f"Large block '{block.label}': {len(block.value)} chars")
```

### Efficient Archival Search

```python
# Efficient: Specific, targeted queries
results = client.agents.memory.archival.search(
    agent_id=agent_id,
    query="customer payment preferences credit card",
    limit=5,
    threshold=0.8
)

# Inefficient: Vague, broad queries
results = client.agents.memory.archival.search(
    agent_id=agent_id,
    query="customer",  # Too broad
    limit=100,         # Too many results
    threshold=0.3      # Too low threshold
)
```

## Next Steps

- [Agent API](../agents/index.md) - Agent management operations
- [Tools API](../tools/index.md) - Tool integration
- [Memory Patterns](../../03-patterns/memory-patterns.md) - Advanced memory strategies

## See Also

- [Memory Concepts](../../01-core-concepts/key-abstractions.md#memory) - Understanding memory types
- [Data Flow](../../01-core-concepts/data-flow.md#memory-data-flow) - How memory works internally
- [Common Mistakes](../../05-gotchas/common-mistakes.md#memory-management-mistakes) - Memory pitfalls