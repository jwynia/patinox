# Minimal Example

> The smallest working code that demonstrates Letta's core value

## 30-Second Agent Setup

This example shows the absolute minimum code to create and interact with a Letta agent:

```python
from letta_client import CreateBlock, Letta, MessageCreate

# Connect to server (assumes `letta server` is running)
client = Letta(base_url="http://localhost:8283")

# Create agent with memory
agent = client.agents.create(
    memory_blocks=[
        CreateBlock(label="human", value="Name: Sarah"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)

# Send message and get response
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Hello! What's my name?")]
)

print(f"Agent: {response.messages[-1].content}")
# Output: "Hello Sarah! Your name is Sarah, as I can see from my memory."

# Cleanup
client.agents.delete(agent.id)
```

## What This Example Demonstrates

### Core Value Proposition
- **Stateful Memory**: The agent remembers "Sarah" across conversations
- **Simple API**: Three lines to create, message, and cleanup
- **LLM Agnostic**: Works with OpenAI, Anthropic, local models, etc.

### Key Components
1. **Client Connection**: `Letta(base_url="...")` 
2. **Agent Creation**: `client.agents.create()` with memory blocks
3. **Message Exchange**: `client.agents.messages.create()`
4. **Persistent State**: Agent retains memory between interactions

## Running the Example

### Prerequisites
```bash
# Terminal 1: Start the server
letta server

# Terminal 2: Run the example
python minimal_example.py
```

### Expected Output
```
Agent: Hello Sarah! Your name is Sarah, as I can see from my memory. How can I help you today?
```

## Common Variations

### Using Local Models
```python
# With Ollama
agent = client.agents.create(
    memory_blocks=[CreateBlock(label="human", value="Name: Sarah")],
    model="ollama/llama3",
    embedding="ollama/mxbai-embed-large",
)
```

### Multiple Memory Blocks
```python
agent = client.agents.create(
    memory_blocks=[
        CreateBlock(label="human", value="Name: Sarah, Age: 28"),
        CreateBlock(label="persona", value="Helpful AI assistant"),
        CreateBlock(label="context", value="Customer support conversation"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)
```

### With Custom Tools
```python
def get_weather(location: str) -> str:
    """Get current weather for a location."""
    return f"It's sunny in {location}"

# Create tool and agent
tool = client.tools.upsert_from_function(func=get_weather)
agent = client.agents.create(
    memory_blocks=[CreateBlock(label="human", value="Name: Sarah")],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
    tool_ids=[tool.id],
)
```

## Error Handling

```python
from letta_client.core.api_error import ApiError

try:
    response = client.agents.messages.create(
        agent_id=agent.id,
        messages=[MessageCreate(role="user", content="Hello")]
    )
except ApiError as e:
    print(f"API Error: {e.status_code} - {e.message}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Comparison to Other Frameworks

### Traditional Chatbots
```python
# Traditional approach (stateless)
import openai
response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello"}]
)
# ❌ No memory between calls
```

### Letta Approach
```python
# Letta approach (stateful)
response = client.agents.messages.create(
    agent_id=agent.id,  # Persistent agent
    messages=[MessageCreate(role="user", content="Hello")]
)
# ✅ Agent remembers context across conversations
```

## Next Steps

- [Import Patterns](import-patterns.md) - How to import and configure Letta
- [API Reference](../02-api-reference/index.md) - Detailed method documentation
- [Memory Management](../01-core-concepts/key-abstractions.md) - Understanding memory blocks

## See Also

- [Agent Creation Patterns](../03-patterns/common-use-cases/basic-agent-setup.md)
- [Tool Integration](../03-patterns/common-use-cases/tool-integration.md)
- [🔗 Live ADE Demo](https://app.letta.com)