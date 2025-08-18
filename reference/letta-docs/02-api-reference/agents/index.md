# Agents API

> Create, manage, and interact with Letta agents

## Overview

The Agents API is the core interface for creating and managing stateful agents. Agents maintain persistent memory, can execute tools, and provide consistent conversational experiences.

## Quick Start

```python
from letta_client import Letta, CreateBlock, MessageCreate

client = Letta(base_url="http://localhost:8283")

# Create agent
agent = client.agents.create(
    name="support-agent",
    memory_blocks=[
        CreateBlock(label="human", value="Customer: Sarah"),
        CreateBlock(label="persona", value="Helpful support agent"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)

# Send message
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Hello!")]
)
```

## Agent Operations

### Create Agent

Create a new agent with specified configuration.

```python
def create(
    self,
    name: Optional[str] = None,
    memory_blocks: Optional[List[CreateBlock]] = None,
    model: str,
    embedding: str,
    system_prompt: Optional[str] = None,
    tool_ids: Optional[List[str]] = None,
    include_base_tools: bool = True,
    tool_rules: Optional[List[ToolRule]] = None,
    **kwargs
) -> AgentState:
    """Create a new agent with specified configuration.
    
    Args:
        name: Human-readable agent name (auto-generated if None)
        memory_blocks: Initial memory blocks for the agent
        model: LLM model identifier (e.g., "openai/gpt-4o-mini")
        embedding: Embedding model for semantic search
        system_prompt: Custom system instructions (optional)
        tool_ids: List of tool IDs to attach to the agent
        include_base_tools: Whether to include default tools
        tool_rules: Constraints on tool usage (optional)
        
    Returns:
        AgentState: Created agent configuration and state
        
    Raises:
        ApiError: If model not available or invalid configuration
        ValidationError: If parameters fail validation
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `name` | str | No | auto-generated | 1-64 chars, alphanumeric + hyphens | Human-readable agent name |
| `memory_blocks` | List[CreateBlock] | No | [] | Max 20 blocks | Initial memory blocks |
| `model` | str | **Yes** | None | Must be available model | LLM model handle |
| `embedding` | str | **Yes** | None | Must be available model | Embedding model handle |
| `system_prompt` | str | No | None | Max 8192 chars | Custom system instructions |
| `tool_ids` | List[str] | No | [] | Max 50 tools, valid UUIDs | Tools to attach |
| `include_base_tools` | bool | No | True | N/A | Include default tools |
| `tool_rules` | List[ToolRule] | No | [] | Max 20 rules | Tool usage constraints |

#### Available Models

Use `client.models.list_llms()` and `client.models.list_embedding_models()` to get current options.

**Common LLM Models:**
- `openai/gpt-4o-mini` - Fast, cost-effective
- `openai/gpt-4o` - Most capable
- `anthropic/claude-3-5-sonnet` - Strong reasoning
- `ollama/llama3` - Local deployment

**Common Embedding Models:**
- `openai/text-embedding-3-small` - Fast, good quality
- `openai/text-embedding-3-large` - Higher quality
- `ollama/mxbai-embed-large` - Local deployment

**Example:**
```python
agent = client.agents.create(
    name="customer-service",
    memory_blocks=[
        CreateBlock(label="human", value="VIP Customer: John Doe"),
        CreateBlock(label="persona", value="Professional support agent"),
        CreateBlock(label="context", value="Handling billing inquiries"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
    system_prompt="You are a customer service agent. Be helpful and professional.",
    tool_ids=[search_tool.id, email_tool.id],
    tool_rules=[
        TerminalToolRule(tool_name="send_email"),  # Stop after sending email
    ]
)
```

### Retrieve Agent

Get agent configuration and current state.

```python
def retrieve(agent_id: str) -> AgentState
```

**Example:**
```python
agent = client.agents.retrieve(agent_id="agent_123")
print(f"Agent: {agent.name}")
print(f"Tools: {[tool.name for tool in agent.tools]}")
print(f"Model: {agent.llm_config.model}")
```

### Update Agent

Modify agent configuration.

```python
def update(
    agent_id: str,
    name: str = None,
    system_prompt: str = None,
    model: str = None,
    embedding: str = None,
    **kwargs
) -> AgentState
```

**Example:**
```python
updated_agent = client.agents.update(
    agent_id=agent.id,
    name="senior-support-agent",
    system_prompt="You are a senior customer service agent with escalation authority."
)
```

### List Agents

Get all agents for the current user.

```python
def list(
    name: str = None,
    limit: int = 50,
    cursor: str = None
) -> List[AgentState]
```

**Example:**
```python
# List all agents
all_agents = client.agents.list()

# Filter by name
support_agents = client.agents.list(name="support")

# Pagination
first_page = client.agents.list(limit=10)
next_page = client.agents.list(limit=10, cursor=first_page[-1].id)
```

### Delete Agent

Permanently delete an agent and all associated data.

```python
def delete(agent_id: str) -> None
```

**Example:**
```python
client.agents.delete(agent_id="agent_123")
```

## Messaging

### Send Message

Send a message to an agent and get response.

```python
def messages.create(
    agent_id: str,
    messages: List[MessageCreate],
    stream: bool = False,
    **kwargs
) -> LettaResponse
```

**Parameters:**
- `agent_id`: ID of the target agent
- `messages`: List of messages to send
- `stream`: Whether to stream the response

**Example:**
```python
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[
        MessageCreate(
            role="user",
            content="What's the status of order #12345?"
        )
    ]
)

print(f"Agent response: {response.messages[-1].content}")
print(f"Tokens used: {response.usage.total_tokens}")
```

### Streaming Messages

Get real-time response streaming.

```python
def messages.create_stream(
    agent_id: str,
    messages: List[MessageCreate],
    **kwargs
) -> Iterator[StreamingChunk]
```

**Example:**
```python
stream = client.agents.messages.create_stream(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Tell me a story")]
)

for chunk in stream:
    if hasattr(chunk, 'choices') and chunk.choices:
        delta = chunk.choices[0].delta
        if delta.content:
            print(delta.content, end='', flush=True)
```

### List Messages

Get conversation history for an agent.

```python
def messages.list(
    agent_id: str,
    limit: int = 100,
    start_date: str = None,
    end_date: str = None,
    query: str = None
) -> List[Message]
```

**Example:**
```python
# Get recent messages
recent = client.agents.messages.list(
    agent_id=agent.id,
    limit=20
)

# Search messages
search_results = client.agents.messages.list(
    agent_id=agent.id,
    query="order status",
    start_date="2024-01-01"
)
```

## Memory Management

### Get Memory

Retrieve agent's memory state.

```python
def memory.retrieve(agent_id: str) -> Memory
```

**Example:**
```python
memory = client.agents.memory.retrieve(agent_id=agent.id)
print(f"Core memory blocks: {len(memory.blocks)}")
print(f"Message count: {len(memory.messages)}")
```

### Update Memory Block

Modify a specific memory block.

```python
def memory.update_block(
    agent_id: str,
    block_id: str,
    value: str
) -> Block
```

**Example:**
```python
updated_block = client.agents.memory.update_block(
    agent_id=agent.id,
    block_id="human_block_id",
    value="VIP Customer: John Doe (Account Manager: Sarah)"
)
```

### Create Memory Block

Add a new memory block.

```python
def memory.create_block(
    agent_id: str,
    label: str,
    value: str
) -> Block
```

**Example:**
```python
new_block = client.agents.memory.create_block(
    agent_id=agent.id,
    label="preferences",
    value="Prefers email communication, morning appointments"
)
```

## Tool Management

### Attach Tool

Add a tool to an agent.

```python
def tools.attach(agent_id: str, tool_id: str) -> None
```

**Example:**
```python
client.agents.tools.attach(
    agent_id=agent.id,
    tool_id=weather_tool.id
)
```

### Detach Tool

Remove a tool from an agent.

```python
def tools.detach(agent_id: str, tool_id: str) -> None
```

**Example:**
```python
client.agents.tools.detach(
    agent_id=agent.id,
    tool_id=deprecated_tool.id
)
```

### List Agent Tools

Get all tools available to an agent.

```python
def tools.list(agent_id: str) -> List[Tool]
```

**Example:**
```python
agent_tools = client.agents.tools.list(agent_id=agent.id)
print(f"Available tools: {[tool.name for tool in agent_tools]}")
```

## File Management

### Attach File

Add a file to an agent's knowledge base.

```python
def files.attach(agent_id: str, file_id: str) -> None
```

**Example:**
```python
# Upload file first
file = client.files.upload_file(
    file_path="product_manual.pdf",
    description="Product documentation"
)

# Attach to agent
client.agents.files.attach(
    agent_id=agent.id,
    file_id=file.id
)
```

### Detach File

Remove a file from an agent.

```python
def files.detach(agent_id: str, file_id: str) -> None
```

### List Agent Files

Get all files attached to an agent.

```python
def files.list(agent_id: str) -> List[FileMetadata]
```

## Advanced Operations

### Clone Agent

Create a copy of an existing agent.

```python
def clone(
    agent_id: str,
    name: str = None
) -> AgentState
```

**Example:**
```python
cloned_agent = client.agents.clone(
    agent_id=original_agent.id,
    name="support-agent-v2"
)
```

### Export Agent

Export agent configuration and memory.

```python
def export(agent_id: str) -> dict
```

**Example:**
```python
agent_data = client.agents.export(agent_id=agent.id)
# Save to file for backup or transfer
```

### Import Agent

Create agent from exported data.

```python
def import_agent(data: dict) -> AgentState
```

## Response Schemas

### AgentState

```python
class AgentState:
    id: str                          # Agent UUID
    name: str                        # Agent name
    user_id: str                     # Owner user ID
    created_at: datetime            # Creation timestamp
    updated_at: datetime            # Last modification
    llm_config: LLMConfig           # LLM configuration
    embedding_config: EmbeddingConfig # Embedding configuration
    memory: Memory                   # Agent memory state
    tools: List[Tool]               # Available tools
    system_prompt: str              # System instructions
    agent_type: str                 # Agent implementation type
```

### LettaResponse

```python
class LettaResponse:
    messages: List[Message]         # Agent's response messages
    usage: UsageStatistics         # Token usage information
    agent_state: AgentState        # Updated agent configuration
    
class Message:
    id: str                        # Message UUID
    role: str                      # "user", "assistant", "system"
    content: str                   # Message text content
    tool_calls: Optional[List[ToolCall]]  # Tools called by agent
    created_at: datetime           # Message timestamp
    
class UsageStatistics:
    prompt_tokens: int             # Input tokens used
    completion_tokens: int         # Output tokens generated
    total_tokens: int              # Total tokens used
    step_count: int                # Number of agent steps
```

## Error Reference

### HTTP Status Codes

| Code | Error Type | Cause | Solution |
|------|------------|-------|----------|
| 400 | Bad Request | Invalid parameters | Check parameter validation rules |
| 401 | Unauthorized | Missing/invalid token | Set valid API token |
| 403 | Forbidden | Access denied | Check user permissions |
| 404 | Not Found | Resource doesn't exist | Verify agent/resource ID |
| 409 | Conflict | Name already exists | Use different name |
| 422 | Validation Error | Parameter validation failed | Check constraints table |
| 429 | Rate Limited | Too many requests | Implement backoff/retry |
| 500 | Server Error | Internal server error | Check server status, retry |

### Common Error Patterns

```python
from letta_client.core.api_error import ApiError

try:
    agent = client.agents.retrieve("invalid_id")
except ApiError as e:
    if e.status_code == 404:
        print("Agent not found")
    elif e.status_code == 403:
        print("Access denied - check user permissions")
    elif e.status_code == 422:
        print(f"Validation error: {e.message}")
    elif e.status_code == 429:
        print("Rate limited - implement backoff")
    else:
        print(f"API error {e.status_code}: {e.message}")
```

### Validation Error Details

```python
# Model validation
try:
    agent = client.agents.create(model="invalid_model", embedding="invalid_embed")
except ApiError as e:
    if e.status_code == 422:
        if "model" in e.message.lower():
            available = client.models.list_llms()
            print(f"Available models: {[m.handle for m in available]}")
        if "embedding" in e.message.lower():
            available = client.models.list_embedding_models()
            print(f"Available embeddings: {[m.handle for m in available]}")

# Memory block validation
try:
    agent = client.agents.create(
        model="openai/gpt-4o-mini",
        embedding="openai/text-embedding-3-small",
        memory_blocks=[CreateBlock(label="", value="")]  # Invalid empty label
    )
except ApiError as e:
    if "label" in e.message.lower():
        print("Memory block labels cannot be empty")

# Name validation
try:
    agent = client.agents.create(
        name="a" * 100,  # Too long
        model="openai/gpt-4o-mini",
        embedding="openai/text-embedding-3-small"
    )
except ApiError as e:
    if "name" in e.message.lower():
        print("Agent name must be 1-64 characters")
```

## Performance Tips

### Efficient Agent Creation
```python
# Pre-create tools and reuse
common_tools = [
    client.tools.upsert_from_function(func=get_weather),
    client.tools.upsert_from_function(func=send_email),
]

# Create agents with same tools
for config in agent_configs:
    agent = client.agents.create(
        tool_ids=[tool.id for tool in common_tools],
        **config
    )
```

### Memory Management
```python
# Monitor memory usage
memory = client.agents.memory.retrieve(agent_id)
if len(memory.messages) > 1000:
    # Consider manual summarization or cleanup
    pass
```

## Next Steps

- [Memory API](../memory/index.md) - Detailed memory management
- [Tools API](../tools/index.md) - Creating and managing tools
- [Agent Patterns](../../03-patterns/common-use-cases/agent-patterns.md) - Common usage patterns

## See Also

- [AgentState Schema](../schemas/agent.md) - Agent configuration format
- [Message Schema](../schemas/message.md) - Message format specification
- [Tool Integration](../../03-patterns/common-use-cases/tool-integration.md) - Tool usage examples