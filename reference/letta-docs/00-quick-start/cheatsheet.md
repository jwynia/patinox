# API Cheatsheet

> Most common functions and methods with copy-pasteable snippets

## Client Setup

```python
from letta_client import Letta, CreateBlock, MessageCreate
client = Letta(base_url="http://localhost:8283")
```

## Agent Operations

### Create Agent
```python
agent = client.agents.create(
    name="my_agent",
    memory_blocks=[
        CreateBlock(label="human", value="Name: Sarah"),
        CreateBlock(label="persona", value="Helpful assistant"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)
```

### Send Message
```python
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Hello!")]
)
print(response.messages[-1].content)
```

### List Agents
```python
agents = client.agents.list()  # All agents
agents = client.agents.list(name="specific_name")  # By name
agents = client.agents.list(limit=10)  # With limit
```

### Get Agent
```python
agent = client.agents.retrieve(agent_id="agent_id")
```

### Update Agent
```python
client.agents.update(
    agent_id=agent.id,
    name="new_name",
    system="Updated system prompt"
)
```

### Delete Agent
```python
client.agents.delete(agent_id=agent.id)
```

## Memory Management

### Get Memory
```python
memory = client.agents.memory.retrieve(agent_id=agent.id)
blocks = memory.blocks
```

### Update Memory Block
```python
client.agents.memory.update_block(
    agent_id=agent.id,
    block_id="block_id",
    value="Updated content"
)
```

### Create Memory Block
```python
block = client.agents.memory.create_block(
    agent_id=agent.id,
    label="new_block",
    value="Block content"
)
```

## Tool Management

### Create Tool from Function
```python
def my_tool(param: str) -> str:
    """Tool description."""
    return f"Result: {param}"

tool = client.tools.upsert_from_function(func=my_tool)
```

### List Tools
```python
tools = client.tools.list()
base_tools = client.tools.list(name="send_message")
```

### Attach Tool to Agent
```python
client.agents.tools.attach(agent_id=agent.id, tool_id=tool.id)
```

### Detach Tool from Agent
```python
client.agents.tools.detach(agent_id=agent.id, tool_id=tool.id)
```

## File and Source Management

### Upload File
```python
file = client.files.upload_file(
    file_path="/path/to/document.pdf",
    description="Important document"
)
```

### Attach File to Agent
```python
client.agents.files.attach(agent_id=agent.id, file_id=file.id)
```

### Create Source from Text
```python
source = client.sources.create(
    name="knowledge_base",
    description="Company knowledge",
    text="Important information here..."
)
```

### Attach Source to Agent
```python
client.agents.sources.attach(agent_id=agent.id, source_id=source.id)
```

## Message History

### Get Messages
```python
messages = client.agents.messages.list(
    agent_id=agent.id,
    limit=50
)
```

### Search Messages
```python
messages = client.agents.messages.list(
    agent_id=agent.id,
    query="search term",
    start_date="2024-01-01",
    end_date="2024-12-31"
)
```

## Archival Memory

### Insert into Archival Memory
```python
client.agents.memory.archival.insert(
    agent_id=agent.id,
    content="Important information to remember"
)
```

### Search Archival Memory
```python
results = client.agents.memory.archival.search(
    agent_id=agent.id,
    query="search term",
    limit=10
)
```

## Server Configuration

### List Models
```python
llm_models = client.models.list_llms()
embedding_models = client.models.list_embedding_models()
```

### Check Server Health
```python
health = client.health.get_health()
print(f"Status: {health.status}")
```

## Streaming Responses

```python
from letta_client import MessageCreate

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

## Error Handling

```python
from letta_client.core.api_error import ApiError

try:
    response = client.agents.messages.create(
        agent_id="invalid_id",
        messages=[MessageCreate(role="user", content="Hello")]
    )
except ApiError as e:
    print(f"API Error {e.status_code}: {e.message}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Common Patterns

### Agent with Custom Tools
```python
# Define tool
def get_time() -> str:
    """Get current time."""
    from datetime import datetime
    return datetime.now().isoformat()

# Create tool and agent
tool = client.tools.upsert_from_function(func=get_time)
agent = client.agents.create(
    memory_blocks=[CreateBlock(label="human", value="User")],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
    tool_ids=[tool.id],
)
```

### Agent with Knowledge Base
```python
# Upload document
file = client.files.upload_file(file_path="knowledge.pdf")

# Create agent with file
agent = client.agents.create(
    memory_blocks=[CreateBlock(label="human", value="User")],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)

# Attach file
client.agents.files.attach(agent_id=agent.id, file_id=file.id)
```

### Conversation Loop
```python
while True:
    user_input = input("You: ")
    if user_input.lower() in ['quit', 'exit']:
        break
    
    response = client.agents.messages.create(
        agent_id=agent.id,
        messages=[MessageCreate(role="user", content=user_input)]
    )
    
    print(f"Agent: {response.messages[-1].content}")
```

## Environment Variables Quick Reference

```bash
# LLM Providers
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export OLLAMA_BASE_URL="http://localhost:11434"

# Database
export LETTA_PG_URI="postgresql://user:pass@host:5432/db"

# Server
export LETTA_SERVER_URL="http://localhost:8283"
export SECURE=true
export LETTA_SERVER_PASSWORD="password"
```

## Quick Debugging

```python
# Check connection
try:
    health = client.health.get_health()
    print("✅ Server connected")
except:
    print("❌ Server connection failed")

# Check agent exists
try:
    agent = client.agents.retrieve(agent_id="agent_id")
    print(f"✅ Agent {agent.name} found")
except:
    print("❌ Agent not found")

# Check models available
models = client.models.list_llms()
print(f"Available models: {[m.handle for m in models]}")
```

## See Also

- [Minimal Example](minimal-example.md) - Full working example
- [API Reference](../02-api-reference/index.md) - Complete documentation
- [Common Use Cases](../03-patterns/common-use-cases/index.md) - Real-world examples