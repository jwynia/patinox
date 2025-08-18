# Common Use Cases

> Real-world usage scenarios and implementation patterns for Letta agents

## Overview

This section covers practical implementation patterns for common agent use cases. Each pattern includes complete working examples, best practices, and common pitfalls.

## Use Case Categories

### Conversational Agents
- [Customer Support Agent](customer-support-agent.md) - Help desk and support ticket handling
- [Personal Assistant](personal-assistant.md) - Scheduling, reminders, and task management
- [Domain Expert](domain-expert.md) - Specialized knowledge consultation
- [Multi-language Support](multi-language-agent.md) - Cross-language communication

### Knowledge Workers  
- [Document Q&A Agent](document-qa-agent.md) - Query large document collections
- [Research Assistant](research-assistant.md) - Gather and synthesize information
- [Code Review Agent](code-review-agent.md) - Automated code analysis and feedback
- [Data Analysis Agent](data-analysis-agent.md) - Process and interpret datasets

### Business Automation
- [Sales Qualification](sales-qualification.md) - Lead scoring and qualification
- [Order Processing](order-processing.md) - E-commerce order management
- [Content Generation](content-generation.md) - Marketing and content creation
- [Workflow Orchestration](workflow-orchestration.md) - Multi-step business processes

### Integration Patterns
- [API Integration Agent](api-integration.md) - External service coordination
- [Database Agent](database-agent.md) - Data query and manipulation
- [Notification Agent](notification-agent.md) - Multi-channel messaging
- [Monitoring Agent](monitoring-agent.md) - System health and alerting

## Quick Examples

### Customer Support Agent
```python
from letta_client import Letta, CreateBlock, MessageCreate

client = Letta(base_url="http://localhost:8283")

# Create support agent with knowledge base
support_agent = client.agents.create(
    name="customer-support",
    memory_blocks=[
        CreateBlock(label="persona", value="Professional customer support agent"),
        CreateBlock(label="guidelines", value="Always be helpful, ask for order numbers when needed"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
    tool_ids=[lookup_order_tool.id, send_email_tool.id],
)

# Attach knowledge base
client.agents.sources.attach(
    agent_id=support_agent.id,
    source_id=support_kb_source.id
)

# Handle support request
response = client.agents.messages.create(
    agent_id=support_agent.id,
    messages=[MessageCreate(
        role="user",
        content="I need to return an item from order #12345"
    )]
)
```

### Document Q&A Agent
```python
# Upload document
document = client.files.upload_file(
    file_path="company_handbook.pdf",
    description="Employee handbook"
)

# Create Q&A agent
qa_agent = client.agents.create(
    name="handbook-qa",
    memory_blocks=[
        CreateBlock(label="persona", value="Helpful HR assistant"),
        CreateBlock(label="context", value="Answer questions about company policies"),
    ],
    model="openai/gpt-4o-mini",
    embedding="openai/text-embedding-3-small",
)

# Attach document
client.agents.files.attach(agent_id=qa_agent.id, file_id=document.id)

# Ask questions
response = client.agents.messages.create(
    agent_id=qa_agent.id,
    messages=[MessageCreate(
        role="user", 
        content="What's the vacation policy?"
    )]
)
```

### Personal Assistant Agent
```python
# Define calendar tool
def schedule_meeting(title: str, date: str, duration: int = 60) -> str:
    """Schedule a meeting on the calendar."""
    # Implementation would integrate with calendar API
    return f"Scheduled '{title}' for {date}, {duration} minutes"

# Create assistant with tools
calendar_tool = client.tools.upsert_from_function(func=schedule_meeting)

assistant = client.agents.create(
    name="personal-assistant",
    memory_blocks=[
        CreateBlock(label="human", value="User: John, Timezone: PST"),
        CreateBlock(label="persona", value="Efficient personal assistant"),
        CreateBlock(label="preferences", value="Prefer morning meetings, lunch at noon"),
    ],
    tool_ids=[calendar_tool.id],
    model="openai/gpt-4o-mini",
)

# Schedule meeting
response = client.agents.messages.create(
    agent_id=assistant.id,
    messages=[MessageCreate(
        role="user",
        content="Schedule a team meeting for tomorrow at 2 PM"
    )]
)
```

## Implementation Patterns

### Stateful Conversation Management
```python
# Pattern: Maintaining context across sessions
class ConversationManager:
    def __init__(self, agent_id: str):
        self.agent_id = agent_id
        self.client = Letta(base_url="http://localhost:8283")
    
    def continue_conversation(self, user_input: str):
        # Agent automatically maintains context from previous interactions
        response = self.client.agents.messages.create(
            agent_id=self.agent_id,
            messages=[MessageCreate(role="user", content=user_input)]
        )
        return response.messages[-1].content
    
    def get_conversation_summary(self):
        # Get recent context
        messages = self.client.agents.messages.list(
            agent_id=self.agent_id,
            limit=50
        )
        return f"Conversation has {len(messages)} messages"
```

### Knowledge-Enhanced Responses
```python
# Pattern: Agent with domain expertise
def create_expert_agent(domain: str, knowledge_files: List[str]):
    agent = client.agents.create(
        name=f"{domain}-expert",
        memory_blocks=[
            CreateBlock(
                label="persona", 
                value=f"Expert in {domain} with access to authoritative sources"
            ),
            CreateBlock(
                label="context", 
                value=f"Provide accurate, detailed answers about {domain}"
            ),
        ],
        model="openai/gpt-4o-mini",
        embedding="openai/text-embedding-3-small",
    )
    
    # Attach knowledge sources
    for file_path in knowledge_files:
        file = client.files.upload_file(file_path=file_path)
        client.agents.files.attach(agent_id=agent.id, file_id=file.id)
    
    return agent

# Usage
legal_agent = create_expert_agent(
    domain="legal",
    knowledge_files=["contract_law.pdf", "regulations.pdf"]
)
```

### Tool-Powered Automation
```python
# Pattern: Agent with multiple capabilities
def create_automation_agent(tools: List[callable]):
    # Convert functions to tools
    tool_ids = []
    for func in tools:
        tool = client.tools.upsert_from_function(func=func)
        tool_ids.append(tool.id)
    
    agent = client.agents.create(
        name="automation-agent",
        memory_blocks=[
            CreateBlock(
                label="persona", 
                value="Efficient automation assistant"
            ),
        ],
        tool_ids=tool_ids,
        model="openai/gpt-4o-mini",
    )
    
    return agent

# Define automation tools
def send_slack_message(channel: str, message: str) -> str:
    """Send message to Slack channel."""
    # Slack API integration
    return f"Sent to #{channel}: {message}"

def update_database(table: str, record_id: str, data: dict) -> str:
    """Update database record."""
    # Database update logic
    return f"Updated {table} record {record_id}"

# Create agent with automation tools
automation_agent = create_automation_agent([
    send_slack_message,
    update_database,
])
```

## Best Practices by Use Case

### Customer Support
- **Memory Strategy**: Store customer context in core memory, conversation history in archival
- **Tool Selection**: Order lookup, email sending, escalation tools
- **Knowledge Base**: FAQ, policies, product documentation
- **Response Tone**: Professional, empathetic, solution-focused

### Personal Assistant
- **Memory Strategy**: User preferences, calendar context, recurring patterns
- **Tool Selection**: Calendar, email, reminders, external service integrations
- **Knowledge Base**: User's documents, preferences, contact information
- **Response Tone**: Friendly, efficient, proactive

### Document Q&A
- **Memory Strategy**: Query history, user expertise level, document structure
- **Tool Selection**: Search, summarization, citation tools
- **Knowledge Base**: Structured document collections, metadata
- **Response Tone**: Informative, precise, well-cited

### Research Assistant
- **Memory Strategy**: Research goals, methodology, source credibility
- **Tool Selection**: Web search, database access, analysis tools
- **Knowledge Base**: Reference materials, research methodologies
- **Response Tone**: Analytical, thorough, source-conscious

## Performance Considerations

### Memory Management
```python
# Monitor and optimize memory usage
def optimize_agent_memory(agent_id: str):
    memory = client.agents.memory.retrieve(agent_id)
    
    # Check message count
    if len(memory.messages) > 1000:
        print(f"Agent has {len(memory.messages)} messages - consider cleanup")
    
    # Check core memory efficiency
    for block in memory.blocks:
        if len(block.value) > 500:
            print(f"Block '{block.label}' is large - consider summarization")
```

### Tool Usage Optimization
```python
# Efficient tool management
def optimize_agent_tools(agent_id: str):
    tools = client.agents.tools.list(agent_id)
    
    # Remove unused tools
    # Monitor tool usage in messages and remove rarely used tools
    
    # Group related tools
    # Combine similar functionality to reduce choice complexity
```

## Troubleshooting Common Issues

### Agent Not Remembering Context
```python
# Check memory configuration
memory = client.agents.memory.retrieve(agent_id)
print(f"Core memory blocks: {len(memory.blocks)}")
print(f"Message history: {len(memory.messages)}")

# Verify memory blocks are being updated
for block in memory.blocks:
    print(f"{block.label}: {block.value[:100]}...")
```

### Tools Not Being Used
```python
# Verify tool attachment
tools = client.agents.tools.list(agent_id)
print(f"Available tools: {[tool.name for tool in tools]}")

# Check tool descriptions
for tool in tools:
    print(f"{tool.name}: {tool.description}")
```

### Inconsistent Responses
```python
# Check system prompt and core memory
agent = client.agents.retrieve(agent_id)
print(f"System prompt: {agent.system}")

memory = client.agents.memory.retrieve(agent_id)
for block in memory.blocks:
    if block.label == "persona":
        print(f"Persona: {block.value}")
```

## Next Steps

- [Composition Patterns](../composition-patterns.md) - Combining features effectively
- [Error Handling](../error-handling.md) - Robust error management
- [Performance Patterns](../performance-patterns.md) - Optimization strategies

## See Also

- [Agent API Reference](../../02-api-reference/agents/index.md) - Technical documentation
- [Tool Development](../tool-patterns.md) - Creating custom tools
- [Integration Examples](../../04-integration/frameworks/index.md) - Framework integration