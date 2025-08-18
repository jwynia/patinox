# API Reference

> Complete documentation for Letta's APIs and components

## Overview

Letta provides multiple interfaces for building and interacting with stateful agents:

- **Python Client SDK** (`letta_client`): Primary interface for applications
- **REST API**: HTTP endpoints for web applications and integrations
- **CLI**: Command-line interface for development and administration

## Quick Navigation

### **🚀 [Quick Reference Tables](quick-reference.md)** - Structured tables for LLM agents

### Client SDK (Primary Interface)
- [Agents API](agents/index.md) - Create, manage, and interact with agents ✅
- [Memory API](memory/index.md) - Manage agent memory and blocks ✅
- [Tools API](tools/index.md) - Create and manage custom tools ✅
- [Files API](files/index.md) - Upload and attach documents ⏳
- [Sources API](sources/index.md) - Manage knowledge sources ⏳
- [Models API](models/index.md) - Configure LLM and embedding models ⏳

### Core Schemas
- [Agent Schema](schemas/agent.md) - Agent configuration and state
- [Message Schema](schemas/message.md) - Conversation messages
- [Memory Schema](schemas/memory.md) - Memory blocks and archival storage
- [Tool Schema](schemas/tool.md) - Tool definitions and execution results

### Server Components
- [REST Endpoints](server/rest-api.md) - HTTP API documentation
- [Authentication](server/auth.md) - User authentication and authorization
- [WebSocket API](server/websocket.md) - Real-time communication

### Internal APIs
- [Service Managers](services/index.md) - Business logic layer
- [Agent Implementation](agents/letta-agent.md) - Core agent behavior
- [LLM Clients](llm-api/index.md) - Provider integrations

## Installation and Setup

```python
# Install the client
pip install letta

# Import and connect
from letta_client import Letta, CreateBlock, MessageCreate

client = Letta(base_url="http://localhost:8283")
```

## Common Operations

### Create and Message Agent
```python
# Create agent
agent = client.agents.create(
    memory_blocks=[
        CreateBlock(label="human", value="Name: Sarah"),
        CreateBlock(label="persona", value="Helpful assistant"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)

# Send message
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Hello!")]
)

print(response.messages[-1].content)
```

### Tool Management
```python
# Create custom tool
def get_weather(location: str) -> str:
    """Get weather for a location."""
    return f"Sunny in {location}"

tool = client.tools.upsert_from_function(func=get_weather)

# Attach to agent
client.agents.tools.attach(agent_id=agent.id, tool_id=tool.id)
```

### File and Knowledge Management
```python
# Upload file
file = client.files.upload_file(
    file_path="document.pdf",
    description="Important document"
)

# Attach to agent
client.agents.files.attach(agent_id=agent.id, file_id=file.id)
```

## Response Formats

### Standard Response Structure
```python
from letta_client.types import LettaResponse

response = LettaResponse(
    messages=[...],           # Agent's response messages
    usage=UsageStatistics,    # Token usage information
    agent_state=AgentState,   # Updated agent configuration
)
```

### Error Handling
```python
from letta_client.core.api_error import ApiError

try:
    response = client.agents.messages.create(...)
except ApiError as e:
    print(f"Error {e.status_code}: {e.message}")
    # Handle specific error cases
```

## API Conventions

### Naming Patterns
- **Resources**: Plural nouns (`agents`, `tools`, `files`)
- **Operations**: Standard verbs (`create`, `retrieve`, `update`, `delete`, `list`)
- **Parameters**: Snake_case for Python, camelCase for JSON

### Return Value Patterns
- **Single items**: Return the object directly
- **Collections**: Return list with optional pagination
- **Operations**: Return updated state or confirmation

### Callback Styles
```python
# Synchronous (default)
response = client.agents.messages.create(...)

# Streaming
for chunk in client.agents.messages.create_stream(...):
    process_chunk(chunk)

# Async (where supported)
response = await client.agents.messages.acreate(...)
```

## Authentication and Authorization

### API Token Authentication
```python
# With token
client = Letta(
    base_url="http://localhost:8283",
    token="your-api-token"
)
```

### User Context
All operations include user context for multi-tenancy:
```python
# Users can only access their own resources
agents = client.agents.list()  # Only returns current user's agents
```

## Rate Limiting and Quotas

### Default Limits
- **Requests**: 1000/hour per user
- **Agents**: 100 per user
- **Tools**: 1000 per user
- **File uploads**: 100MB per file

### Usage Monitoring
```python
# Check usage
health = client.health.get_health()
print(f"Status: {health.status}")

# Monitor token usage
response = client.agents.messages.create(...)
print(f"Tokens used: {response.usage.total_tokens}")
```

## Versioning

### API Versions
- **v1**: Current stable API (recommended)
- **OpenAI Compatible**: `/openai/v1/chat/completions` for OpenAI SDK compatibility

### Client Versioning
```python
import letta
print(f"Client version: {letta.__version__}")

# Version compatibility check
if letta.__version__ < "0.11.0":
    raise RuntimeError("Letta 0.11.0+ required")
```

## Performance Considerations

### Connection Pooling
```python
# Reuse client instance
client = Letta(base_url="http://localhost:8283")

# Use for multiple operations
for i in range(100):
    response = client.agents.messages.create(...)
```

### Batching Operations
```python
# Create multiple agents efficiently
agents = []
for config in agent_configs:
    agent = client.agents.create(**config)
    agents.append(agent)
```

### Caching
```python
# Cache expensive operations
@lru_cache(maxsize=100)
def get_agent_tools(agent_id):
    return client.agents.tools.list(agent_id)
```

## API Reference Sections

### Core APIs
- [Agents](agents/index.md) - Agent lifecycle and interaction
- [Memory](memory/index.md) - Memory management
- [Tools](tools/index.md) - Tool creation and management
- [Messages](messages/index.md) - Conversation handling

### Data Management
- [Files](files/index.md) - File upload and processing
- [Sources](sources/index.md) - Knowledge base management
- [Blocks](blocks/index.md) - Memory block operations

### Configuration
- [Models](models/index.md) - LLM and embedding configuration
- [Users](users/index.md) - User management
- [Health](health/index.md) - Server status

### Advanced
- [Streaming](streaming/index.md) - Real-time responses
- [WebSocket](websocket/index.md) - Persistent connections
- [Batch Operations](batch/index.md) - Bulk processing

## Next Steps

- [Agent API](agents/index.md) - Start with agent creation
- [Common Use Cases](../03-patterns/common-use-cases/index.md) - Real-world examples
- [Integration Guide](../04-integration/frameworks/index.md) - Framework integration

## See Also

- [🔗 OpenAPI Specification](http://localhost:8283/docs) - Interactive API docs
- [Client SDK Source](https://github.com/letta-ai/letta-client-python) - Python client
- [REST API Examples](../03-patterns/api-usage/index.md) - Practical examples