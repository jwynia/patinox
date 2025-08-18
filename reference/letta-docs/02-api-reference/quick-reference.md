# Quick Reference Tables

> Structured reference for LLM agents and developers

## Core Operations Reference

### Agent Operations

| Operation | Method | Required Parameters | Optional Parameters | Returns | Error Codes |
|-----------|--------|-------------------|-------------------|---------|-------------|
| **Create Agent** | `client.agents.create()` | `model`, `embedding` | `name`, `memory_blocks`, `tool_ids`, `system_prompt`, `include_base_tools`, `tool_rules` | `AgentState` | 400, 422 |
| **Get Agent** | `client.agents.retrieve(agent_id)` | `agent_id` | None | `AgentState` | 404, 403 |
| **Update Agent** | `client.agents.update(agent_id, ...)` | `agent_id` | `name`, `system_prompt`, `model`, `embedding` | `AgentState` | 404, 422 |
| **List Agents** | `client.agents.list()` | None | `name`, `limit`, `cursor` | `List[AgentState]` | 400 |
| **Delete Agent** | `client.agents.delete(agent_id)` | `agent_id` | None | `None` | 404, 403 |

### Message Operations

| Operation | Method | Required Parameters | Optional Parameters | Returns | Error Codes |
|-----------|--------|-------------------|-------------------|---------|-------------|
| **Send Message** | `client.agents.messages.create()` | `agent_id`, `messages` | `stream` | `LettaResponse` | 404, 400, 429 |
| **Stream Message** | `client.agents.messages.create_stream()` | `agent_id`, `messages` | None | `Iterator[StreamChunk]` | 404, 400, 429 |
| **List Messages** | `client.agents.messages.list()` | `agent_id` | `limit`, `cursor`, `start_date`, `end_date`, `query` | `List[Message]` | 404, 400 |

### Memory Operations

| Operation | Method | Required Parameters | Optional Parameters | Returns | Error Codes |
|-----------|--------|-------------------|-------------------|---------|-------------|
| **Get Memory** | `client.agents.memory.retrieve(agent_id)` | `agent_id` | None | `Memory` | 404, 403 |
| **Update Block** | `client.agents.memory.update_block()` | `agent_id`, `block_id`, `value` | None | `Block` | 404, 422 |
| **Create Block** | `client.agents.memory.create_block()` | `agent_id`, `label`, `value` | None | `Block` | 404, 409, 422 |
| **Delete Block** | `client.agents.memory.delete_block()` | `agent_id`, `block_id` | None | `None` | 404, 422 |
| **Search Archival** | `client.agents.memory.archival.search()` | `agent_id`, `query` | `limit`, `threshold` | `List[Passage]` | 404, 400 |
| **Insert Archival** | `client.agents.memory.archival.insert()` | `agent_id`, `content` | `metadata` | `Passage` | 404, 422 |

### Tool Operations

| Operation | Method | Required Parameters | Optional Parameters | Returns | Error Codes |
|-----------|--------|-------------------|-------------------|---------|-------------|
| **Create Tool** | `client.tools.upsert_from_function(func)` | `func` | `name`, `description`, `tags` | `Tool` | 400, 422 |
| **Get Tool** | `client.tools.retrieve(tool_id)` | `tool_id` | None | `Tool` | 404, 403 |
| **List Tools** | `client.tools.list()` | None | `name`, `tags`, `limit`, `cursor` | `List[Tool]` | 400 |
| **Update Tool** | `client.tools.update()` | `tool_id` | `name`, `description`, `tags` | `Tool` | 404, 422 |
| **Delete Tool** | `client.tools.delete(tool_id)` | `tool_id` | None | `None` | 404, 403 |
| **Attach to Agent** | `client.agents.tools.attach()` | `agent_id`, `tool_id` | None | `None` | 404, 409 |
| **Detach from Agent** | `client.agents.tools.detach()` | `agent_id`, `tool_id` | None | `None` | 404, 422 |
| **List Agent Tools** | `client.agents.tools.list(agent_id)` | `agent_id` | None | `List[Tool]` | 404, 403 |

## Parameter Constraints Reference

### Agent Parameters

| Parameter | Type | Min/Max | Format | Default | Example |
|-----------|------|---------|--------|---------|---------|
| `name` | str | 1-64 chars | alphanumeric + hyphens | auto-generated | "customer-support" |
| `model` | str | Must exist | provider/model | Required | "openai/gpt-4o-mini" |
| `embedding` | str | Must exist | provider/model | Required | "openai/text-embedding-3-small" |
| `system_prompt` | str | 1-8192 chars | any text | None | "You are a helpful assistant" |
| `memory_blocks` | List | 0-20 blocks | CreateBlock objects | [] | `[CreateBlock(label="human", value="...")]` |
| `tool_ids` | List | 0-50 tools | UUID strings | [] | `["tool_123", "tool_456"]` |
| `tool_rules` | List | 0-20 rules | ToolRule objects | [] | `[TerminalToolRule(tool_name="send_email")]` |

### Memory Parameters

| Parameter | Type | Min/Max | Format | Default | Example |
|-----------|------|---------|--------|---------|---------|
| `label` | str | 1-32 chars | alphanumeric + underscores | Required | "human", "persona", "context" |
| `value` | str | 1-2048 chars | any text | Required | "Customer: John Doe - VIP member" |
| `query` | str | 1-512 chars | search text | Required | "customer payment preferences" |
| `limit` | int | 1-100 | positive integer | 10 | 5, 20, 50 |
| `threshold` | float | 0.0-1.0 | decimal | 0.7 | 0.8, 0.9 |

### Tool Parameters

| Parameter | Type | Min/Max | Format | Default | Example |
|-----------|------|---------|--------|---------|---------|
| `name` | str | 1-64 chars | alphanumeric + underscores | func.__name__ | "get_weather", "calculate_tax" |
| `description` | str | 1-512 chars | descriptive text | func.__doc__ | "Get current weather for a location" |
| `tags` | List | 0-10 tags | 1-32 chars each | [] | `["api", "weather", "external"]` |

## Response Schema Reference

### Core Response Types

```python
# Agent State
AgentState {
    id: str                    # UUID
    name: str                  # Human-readable name
    user_id: str              # Owner UUID
    created_at: datetime      # ISO 8601
    updated_at: datetime      # ISO 8601
    llm_config: LLMConfig     # Model configuration
    embedding_config: EmbeddingConfig
    memory: Memory            # Current memory state
    tools: List[Tool]         # Available tools
    system_prompt: str        # System instructions
    agent_type: str           # Implementation type
}

# Message Response
LettaResponse {
    messages: List[Message]   # Agent responses
    usage: UsageStatistics   # Token consumption
    agent_state: AgentState  # Updated agent state
}

# Memory Objects
Memory {
    blocks: List[Block]       # Core memory blocks
    messages: List[Message]   # Recent conversation
    archival_memory: ArchivalMemory
}

Block {
    id: str                   # UUID
    label: str               # Block identifier
    value: str               # Block content
    created_at: datetime     # ISO 8601
    updated_at: datetime     # ISO 8601
}

# Tool Objects
Tool {
    id: str                   # UUID
    name: str                # Tool identifier
    description: str         # Tool description
    parameters: dict         # OpenAI function schema
    tags: List[str]          # Categorization
    created_at: datetime     # ISO 8601
    user_id: str             # Owner UUID
}
```

## Error Code Reference

### HTTP Status Codes

| Code | Name | Common Causes | Quick Fix |
|------|------|---------------|-----------|
| **400** | Bad Request | Invalid JSON, malformed parameters | Check request format |
| **401** | Unauthorized | Missing/invalid API token | Set `LETTA_API_TOKEN` |
| **403** | Forbidden | User lacks permissions | Check user access |
| **404** | Not Found | Invalid agent/tool/resource ID | Verify resource exists |
| **409** | Conflict | Duplicate name, already attached | Use different name |
| **422** | Validation Error | Parameter constraints violated | Check parameter tables above |
| **429** | Rate Limited | Too many requests | Implement exponential backoff |
| **500** | Server Error | Internal server issue | Check server status, retry |

### Validation Error Patterns

| Error Message Pattern | Cause | Solution |
|----------------------|-------|----------|
| `"model not available"` | Invalid model string | Use `client.models.list_llms()` |
| `"embedding not available"` | Invalid embedding string | Use `client.models.list_embedding_models()` |
| `"agent not found"` | Invalid agent_id | Use `client.agents.list()` |
| `"tool not found"` | Invalid tool_id | Use `client.tools.list()` |
| `"block not found"` | Invalid block_id | Use `client.agents.memory.retrieve()` |
| `"name already exists"` | Duplicate agent/tool name | Choose unique name |
| `"content too large"` | Text exceeds limits | Split or summarize content |
| `"invalid function"` | Function missing type hints/docstring | Add proper annotations |

## Common Code Patterns

### Basic Setup
```python
from letta_client import Letta, CreateBlock, MessageCreate

client = Letta(base_url="http://localhost:8283")
```

### Error Handling Pattern
```python
from letta_client.core.api_error import ApiError

try:
    result = client.agents.create(...)
except ApiError as e:
    if e.status_code == 422:
        print(f"Validation error: {e.message}")
    elif e.status_code == 404:
        print("Resource not found")
    else:
        print(f"API error {e.status_code}: {e.message}")
```

### Pagination Pattern
```python
all_items = []
cursor = None

while True:
    page = client.agents.list(limit=50, cursor=cursor)
    all_items.extend(page)
    
    if len(page) < 50:  # Last page
        break
    cursor = page[-1].id
```

### Bulk Operations Pattern
```python
# Create multiple agents efficiently
agent_configs = [
    {"name": "agent1", "model": "openai/gpt-4o-mini", ...},
    {"name": "agent2", "model": "openai/gpt-4o-mini", ...},
]

agents = []
for config in agent_configs:
    try:
        agent = client.agents.create(**config)
        agents.append(agent)
    except ApiError as e:
        print(f"Failed to create {config['name']}: {e.message}")
```

## Model Reference

### Common LLM Models

| Provider | Model Handle | Speed | Cost | Use Case |
|----------|-------------|-------|------|----------|
| OpenAI | `openai/gpt-4o-mini` | Fast | Low | General purpose, development |
| OpenAI | `openai/gpt-4o` | Medium | High | Complex reasoning, production |
| OpenAI | `openai/o1-mini` | Slow | Medium | Deep reasoning, analysis |
| Anthropic | `anthropic/claude-3-5-haiku` | Fast | Low | Simple tasks, high volume |
| Anthropic | `anthropic/claude-3-5-sonnet` | Medium | Medium | Balanced performance |
| Local | `ollama/llama3` | Variable | Free | Privacy, offline use |

### Common Embedding Models

| Provider | Model Handle | Dimensions | Use Case |
|----------|-------------|------------|----------|
| OpenAI | `openai/text-embedding-3-small` | 1536 | General purpose, fast |
| OpenAI | `openai/text-embedding-3-large` | 3072 | High quality, slower |
| Local | `ollama/mxbai-embed-large` | 1024 | Privacy, offline |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `LETTA_SERVER_URL` | No | `http://localhost:8283` | Server endpoint |
| `LETTA_API_TOKEN` | No | None | API authentication token |
| `OPENAI_API_KEY` | Conditional | None | OpenAI model access |
| `ANTHROPIC_API_KEY` | Conditional | None | Anthropic model access |
| `LETTA_PG_URI` | No | SQLite | PostgreSQL connection |

## Usage Limits

### Default Quotas

| Resource | Limit | Scope |
|----------|-------|-------|
| Agents per user | 100 | Per user account |
| Tools per user | 1000 | Per user account |
| Tools per agent | 50 | Per agent |
| Memory blocks per agent | 20 | Per agent |
| Memory block size | 2048 chars | Per block |
| Tool rules per agent | 20 | Per agent |
| API requests | 1000/hour | Per user |
| File upload size | 100MB | Per file |

### Rate Limiting

| Endpoint Pattern | Limit | Window |
|------------------|-------|--------|
| `/v1/agents/**` | 100 requests | 1 minute |
| `/v1/agents/*/messages` | 60 requests | 1 minute |
| `/v1/tools/**` | 200 requests | 1 minute |
| All endpoints | 1000 requests | 1 hour |

## Next Steps

- [Agents API](agents/index.md) - Complete agent operations
- [Memory API](memory/index.md) - Memory management details
- [Tools API](tools/index.md) - Tool development guide

## See Also

- [Cheatsheet](../00-quick-start/cheatsheet.md) - Copy-paste examples
- [Common Mistakes](../05-gotchas/common-mistakes.md) - Error prevention
- [API Coverage](../_meta/validation.md) - Documentation completeness