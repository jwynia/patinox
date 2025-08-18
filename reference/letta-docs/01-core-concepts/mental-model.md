# Mental Model

> The paradigm and philosophy behind Letta's design

## Core Philosophy

### Agents as Persistent Entities
Unlike traditional stateless chatbots, Letta agents are **persistent entities** that:

- **Remember**: Maintain state across conversations
- **Learn**: Accumulate knowledge over time
- **Act**: Execute tools to accomplish tasks
- **Evolve**: Adapt their behavior based on experience

Think of them as **digital colleagues** rather than one-time chat interfaces.

### Memory as the Foundation
Memory is not an afterthought but the **central organizing principle**:

```python
# Traditional approach: stateless
def chat(prompt):
    return llm.complete(prompt)  # No memory of previous calls

# Letta approach: memory-centric  
def chat(agent, prompt):
    agent.memory.add_message(prompt)           # Remember input
    context = agent.memory.get_context()      # Recall relevant info
    response = agent.process(context)          # Generate response
    agent.memory.add_message(response)        # Remember output
    return response
```

## Key Mental Models

### 1. Agent as Autonomous Worker

Think of a Letta agent like hiring a new employee:

```python
# "Hiring" an agent
agent = client.agents.create(
    memory_blocks=[
        CreateBlock(label="role", value="Customer support specialist"),
        CreateBlock(label="knowledge", value="Expert in product returns"),
        CreateBlock(label="personality", value="Patient and helpful")
    ],
    tools=[support_tools]  # "Skills" they can use
)

# "Giving them work"
response = client.agents.messages.create(
    agent_id=agent.id,
    messages=[MessageCreate(role="user", content="Help this customer")]
)
```

**Key Differences from Traditional Bots:**
- Agents **persist** between interactions
- They **accumulate expertise** over time
- They have **consistent personality** and context
- They can **learn from mistakes** and improve

### 2. Memory as Layered Storage

Human-like memory architecture with different types of storage:

#### Working Memory (Core Memory)
Like what's "top of mind" - always accessible and relevant:

```python
core_memory = [
    "Current customer: Sarah Johnson",
    "Issue: Return request for order #1234", 
    "Agent role: Support specialist",
    "Context: Customer is frustrated"
]
```

#### Episodic Memory (Message History)
Like remembering the conversation flow:

```python
conversation = [
    "Customer: I want to return this item",
    "Agent: I can help with that. What's the order number?",
    "Customer: It's #1234",
    "Agent: I see that order. It's within our return window..."
]
```

#### Semantic Memory (Archival)
Like long-term knowledge storage - searchable when needed:

```python
knowledge_base = [
    "Return policy: 30 days from purchase",
    "Common issues: sizing problems with item SKU-789",
    "Sarah's history: 3 previous orders, always satisfied",
    "Escalation procedure: manager approval for >$200 returns"
]
```

### 3. Tools as Capabilities

Tools are like **skills** or **abilities** that agents can develop:

```python
# Basic agent (limited capabilities)
basic_agent = client.agents.create(
    include_base_tools=True  # Just talking and memory
)

# Skilled agent (expanded capabilities)
skilled_agent = client.agents.create(
    tools=[
        search_orders_tool,     # Can look up order info
        process_refund_tool,    # Can issue refunds
        send_email_tool,        # Can send notifications
        escalate_to_human_tool  # Can get human help
    ]
)
```

**Tool Philosophy:**
- **Composable**: Mix and match capabilities
- **Secure**: Sandboxed execution prevents damage
- **Extensible**: Easy to add new skills
- **Contextual**: Used intelligently based on situation

### 4. Context Window as Working Space

The LLM's context window is like a **workspace** where the agent:

1. **Assembles relevant information** from memory
2. **Focuses on the current task** 
3. **Makes decisions** about what to do next

```python
# Agent's "workspace" assembly
workspace = {
    "system_prompt": "You are a helpful customer service agent...",
    "core_memory": agent.memory.get_core_blocks(),
    "recent_conversation": agent.memory.get_recent_messages(50),
    "relevant_knowledge": agent.memory.search_archival(current_topic),
    "available_tools": agent.get_tools()
}

# LLM processes in this "workspace"
response = llm.process(workspace)
```

When the workspace gets full, the agent **summarizes** old information and **archives** details for later retrieval.

## Design Principles

### 1. Transparency
Everything the agent does should be **observable and debuggable**:

```python
# You can inspect all agent state
memory = client.agents.memory.retrieve(agent_id)
messages = client.agents.messages.list(agent_id)
tools = client.agents.tools.list(agent_id)

# Tool calls are logged and visible
for message in messages:
    if message.tool_calls:
        print(f"Agent used tool: {message.tool_calls[0].name}")
```

### 2. Gradualism
Agents should **improve gradually** rather than requiring perfect initial setup:

```python
# Start simple
agent = client.agents.create(basic_config)

# Add capabilities over time
client.agents.tools.attach(agent_id, new_tool_id)
client.agents.sources.attach(agent_id, knowledge_source_id)
client.agents.memory.create_block(agent_id, "learned_preference", value)
```

### 3. User Control
Users should maintain **agency over their agents**:

```python
# Users control memory
client.agents.memory.update_block(agent_id, "persona", "More formal tone")

# Users control capabilities
client.agents.tools.detach(agent_id, risky_tool_id)

# Users control knowledge
client.agents.sources.detach(agent_id, outdated_source_id)
```

### 4. Composability
Components should work together naturally:

```python
# Memory + Tools + Knowledge = Capable Agent
agent = client.agents.create(
    memory_blocks=domain_memory,     # Domain expertise
    tools=domain_tools,              # Domain capabilities  
    sources=domain_knowledge         # Domain information
)
```

## Common Anti-Patterns

### ❌ Treating Agents as Stateless Functions
```python
# Wrong: Using agent like a function
for user_input in inputs:
    response = client.agents.messages.create(agent_id, user_input)
    # Ignoring that agent learns and changes
```

### ✅ Treating Agents as Persistent Entities
```python
# Right: Recognizing agent persistence
agent = client.agents.create(...)
for user_input in inputs:
    response = client.agents.messages.create(agent_id, user_input)
    # Agent gets smarter and more context-aware each iteration
```

### ❌ Overloading Core Memory
```python
# Wrong: Putting everything in core memory
CreateBlock(label="everything", value="Customer name is X, order is Y, policy is Z, history is...")
```

### ✅ Using Appropriate Memory Types
```python
# Right: Using layered memory appropriately  
core_memory = [
    CreateBlock(label="customer", value="Sarah - VIP member"),
    CreateBlock(label="current_issue", value="Return request #1234")
]
# Details go in archival memory for search when needed
```

### ❌ Creating Tools for Everything
```python
# Wrong: Over-tooling
tools = [
    get_weather_tool,     # Not relevant for support agent
    play_music_tool,      # Not relevant for support agent  
    send_email_tool,      # Relevant
    lookup_order_tool     # Relevant
]
```

### ✅ Focused Tool Selection
```python
# Right: Purpose-driven tool selection
support_tools = [
    lookup_order_tool,
    process_refund_tool,
    escalate_to_human_tool,
    send_notification_tool
]
```

## Conceptual Hierarchy

```
Philosophy: Agents as Persistent Digital Colleagues
    ↓
Principle: Memory-Centric Design
    ↓
Architecture: Layered Service Architecture
    ↓
Implementation: Modular, Composable Components
    ↓
Interface: Simple, Powerful APIs
```

## Evolution of Thinking

### Traditional Chatbot Mindset
- **Stateless**: Each interaction is independent
- **Reactive**: Responds to immediate input only
- **Limited**: Can only do what's programmed upfront
- **Ephemeral**: No learning or memory

### Letta Agent Mindset  
- **Stateful**: Maintains continuity across interactions
- **Proactive**: Can initiate actions and use tools
- **Adaptive**: Learns and improves over time
- **Persistent**: Accumulates knowledge and experience

## Real-World Analogies

### Agent as Personal Assistant
```python
# Like hiring a personal assistant who:
# - Remembers your preferences (core memory)
# - Recalls past conversations (message history)  
# - Has access to your files/calendar (sources)
# - Can make calls/send emails (tools)
# - Gets better at helping you over time (learning)
```

### Memory as Filing System
```python
# Core Memory = Desktop (always visible, limited space)
# Message History = Recent documents (chronological, auto-archived)
# Archival Memory = Filing cabinet (searchable, unlimited space)
```

### Tools as Office Equipment
```python
# Basic tools = Phone, computer, email (communication)
# Specialized tools = CRM access, payment processor (domain-specific)
# External tools = Calculator, translator (utility functions)
```

## Next Steps

- [Terminology](terminology.md) - Domain-specific terms and meanings
- [Architecture Overview](architecture-overview.md) - How it all fits together
- [Key Abstractions](key-abstractions.md) - Main entities and relationships

## See Also

- [Agent Development Best Practices](../03-patterns/composition-patterns.md)
- [Memory Management Strategies](../03-patterns/memory-patterns.md)
- [Tool Design Philosophy](../03-patterns/tool-patterns.md)