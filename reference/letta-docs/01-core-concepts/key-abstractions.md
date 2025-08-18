# Key Abstractions

> Main entities, their relationships, and core concepts in Letta

## Primary Entities

### Agent
The central autonomous entity that processes messages and maintains state.

```python
from letta.schemas.agent import AgentState

# Core properties
agent = AgentState(
    id="agent_123",
    name="MyAgent", 
    user_id="user_456",
    llm_config=llm_config,      # LLM provider configuration
    embedding_config=embed_config,  # Embedding model configuration
    memory=memory,              # Agent's memory system
    tools=tools,                # Available tools
    system_prompt="...",        # System instructions
    agent_type="letta_agent"    # Agent implementation type
)
```

**Key Relationships:**
- **User**: Owns agents (multi-tenancy)
- **Memory**: Contains agent's persistent state
- **Tools**: Functions the agent can execute
- **Messages**: Conversation history with the agent

### Memory
Multi-layered memory system for persistent agent state.

#### Core Memory (Block-based)
Structured memory with labeled blocks:

```python
from letta.schemas.block import Block

# Essential agent context
blocks = [
    Block(label="human", value="Name: Sarah, Role: Customer"),
    Block(label="persona", value="Helpful customer service agent"),
    Block(label="context", value="Support ticket #12345")
]
```

#### Archival Memory (Vector-based)
Long-term searchable storage using embeddings:

```python
# Information stored with semantic search capability
archival_memory = [
    "Company policy: Returns accepted within 30 days",
    "Product FAQ: Setup requires admin privileges", 
    "Customer history: Previous issues with billing"
]
```

#### Message History
Conversation context with automatic management:

```python
from letta.schemas.message import Message

messages = [
    Message(role="user", content="Hello"),
    Message(role="assistant", content="Hi! How can I help?"),
    Message(role="user", content="I need help with returns")
]
```

### Tools
Functions that agents can execute to interact with external systems.

```python
from letta.schemas.tool import Tool

def get_order_status(order_id: str) -> str:
    """Retrieve the current status of an order."""
    # Implementation here
    return f"Order {order_id} is shipped"

# Tool automatically generated from function
tool = Tool(
    name="get_order_status",
    description="Retrieve the current status of an order",
    parameters={
        "type": "object",
        "properties": {
            "order_id": {"type": "string", "description": "Order ID to check"}
        }
    }
)
```

**Tool Types:**
- **Built-in Tools**: Core functionality (send_message, memory operations)
- **Custom Tools**: User-defined Python functions
- **External Tools**: MCP, Composio, API integrations
- **File Tools**: Document processing and search

### User
Actor in the system with ownership of resources.

```python
from letta.schemas.user import User

user = User(
    id="user_123",
    name="john_doe",
    # All resources belong to this user
)
```

**User Owns:**
- Agents and their configurations
- Tools and custom functions
- Files and knowledge sources
- Message history and memories

### Source
Knowledge bases that agents can access.

```python
from letta.schemas.source import Source

# Document-based knowledge
source = Source(
    id="source_123",
    name="company_docs",
    description="Internal company documentation",
    embedding_config=embed_config,
    # Contains processed passages for search
)
```

**Source Types:**
- **File Sources**: PDFs, text files, documents
- **Text Sources**: Direct text input
- **Web Sources**: Crawled web content
- **Database Sources**: Structured data

## Relationships Between Entities

### Entity Relationship Diagram

```
User
├── Agents (1:many)
│   ├── Memory
│   │   ├── Blocks (1:many)
│   │   ├── Messages (1:many) 
│   │   └── Archival Passages (1:many)
│   ├── Tools (many:many)
│   └── Sources (many:many)
├── Tools (1:many)
├── Sources (1:many)
└── Files (1:many)
```

### Key Relationships

#### Agent ↔ Memory
One-to-one relationship where each agent has exactly one memory instance:

```python
# Agent's memory contains all persistent state
agent.memory.blocks          # Core memory blocks
agent.memory.messages        # Conversation history
agent.memory.archival_memory # Long-term searchable storage
```

#### Agent ↔ Tools (Many-to-Many)
Agents can have multiple tools, tools can be shared across agents:

```python
# Attach existing tool to agent
client.agents.tools.attach(agent_id=agent.id, tool_id=tool.id)

# Create agent with specific tools
agent = client.agents.create(
    tool_ids=[tool1.id, tool2.id],
    include_base_tools=True  # Include default tools
)
```

#### Agent ↔ Sources (Many-to-Many)
Agents can access multiple knowledge sources:

```python
# Attach source to agent for knowledge access
client.agents.sources.attach(agent_id=agent.id, source_id=source.id)

# Agent can search across all attached sources
results = agent.search_archival_memory("query")
```

#### User ↔ Everything (One-to-Many)
All resources are owned by users for multi-tenancy:

```python
# All operations require user context
agent = agent_manager.get_agent_by_id(
    agent_id=agent_id,
    user=current_user  # Ensures user can only access their agents
)
```

## Core Concepts

### Statefulness
Unlike traditional chatbots, Letta agents maintain persistent state:

```python
# State persists across sessions
session1 = client.agents.messages.create(agent_id, [...])
# Agent remembers from session1

session2 = client.agents.messages.create(agent_id, [...]) 
# Agent still has memory from session1
```

### Context Window Management
Automatic handling of LLM context limits:

```python
# Letta automatically manages context window
# 1. Monitors token usage
# 2. Summarizes old messages when limit approached
# 3. Maintains core memory and recent context
# 4. Stores details in archival memory for search
```

### Tool Execution
Safe, sandboxed execution of agent tools:

```python
# Tools execute in isolated environments
def risky_operation():
    """This runs in a sandbox, can't harm the system."""
    import os
    os.system("rm -rf /")  # This would be blocked/sandboxed

# Different execution environments:
# - Local sandbox (default)
# - E2B cloud sandbox
# - Modal serverless functions
```

### Memory Coherence
Maintaining consistent memory across different storage types:

```python
# Memory automatically synchronized
agent.update_core_memory("human", "Name: Sarah (preferred: Sally)")
# This updates:
# 1. Core memory block
# 2. Adds entry to message history  
# 3. Updates embeddings for search consistency
```

## Advanced Abstractions

### Agent Types
Different agent implementations for different use cases:

```python
# Standard conversational agent
letta_agent = LettaAgent(...)

# Voice-enabled agent with audio processing
voice_agent = VoiceAgent(...)

# Temporary agent for one-time tasks
ephemeral_agent = EphemeralAgent(...)

# Batch processing agent for bulk operations
batch_agent = BatchAgent(...)
```

### Memory Strategies
Different approaches to memory management:

```python
# Static buffer: Fixed-size message window
agent = LettaAgent(
    summarizer_mode=SummarizationMode.STATIC_BUFFER,
    message_buffer_limit=100
)

# Partial eviction: Remove less important messages
agent = LettaAgent(
    summarizer_mode=SummarizationMode.PARTIAL_EVICT,
    partial_evict_percentage=0.3
)
```

### Tool Rules
Control agent tool usage with rules:

```python
from letta_client.types import TerminalToolRule, InitToolRule

# Agent stops after calling specific tool
rules = [
    TerminalToolRule(tool_name="place_order"),
    InitToolRule(tool_name="greet_customer")  # Always start with greeting
]

agent = client.agents.create(
    tool_rules=rules,
    # ...
)
```

### Multi-Agent Groups
Coordinated agent systems:

```python
# Round-robin multi-agent system
group = RoundRobinMultiAgent(
    agents=[agent1, agent2, agent3],
    coordination_strategy="round_robin"
)

# Supervisor-worker pattern
group = SupervisorMultiAgent(
    supervisor_agent=supervisor,
    worker_agents=[worker1, worker2]
)
```

## Mental Models for Agents

### Agent as Persistent Assistant
Think of agents like human assistants who:
- Remember previous conversations
- Have access to company knowledge
- Can use tools to accomplish tasks
- Maintain consistent personality/role

### Memory as Human-like System
- **Core Memory**: Like a human's working memory (names, current context)
- **Archival Memory**: Like long-term memory (searchable knowledge)
- **Message History**: Like episodic memory (conversation history)

### Tools as Skills
Tools are like skills or capabilities:
- Built-in skills (talking, remembering)
- Learned skills (custom functions)
- External skills (API integrations)

## Next Steps

- [Data Flow](data-flow.md) - How information moves through the system
- [Mental Model](mental-model.md) - The philosophy behind Letta's design
- [Terminology](terminology.md) - Domain-specific terms and meanings

## See Also

- [Agent API Reference](../02-api-reference/agents/index.md)
- [Memory Management](../03-patterns/memory-patterns.md)
- [Tool Development](../03-patterns/tool-patterns.md)