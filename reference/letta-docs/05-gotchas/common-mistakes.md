# Common Mistakes

> Frequent errors and their solutions when working with Letta

## Overview

This guide covers the most common mistakes developers make when building with Letta, along with practical solutions and prevention strategies.

## Agent Design Mistakes

### ❌ Treating Agents as Stateless Functions

**Mistake:**
```python
# Wrong: Using agent like a stateless function
def get_response(prompt):
    agent = client.agents.create(...)  # Creating new agent each time
    response = client.agents.messages.create(agent_id=agent.id, messages=[...])
    client.agents.delete(agent.id)  # Destroying agent
    return response
```

**Problem:** Loses all benefits of persistent memory and learning.

**Solution:**
```python
# Right: Create agent once, reuse across interactions
class PersistentAgent:
    def __init__(self):
        self.agent = client.agents.create(
            name="persistent-helper",
            memory_blocks=[...],
            model="openai/gpt-4o-mini"
        )
    
    def chat(self, message):
        return client.agents.messages.create(
            agent_id=self.agent.id,
            messages=[MessageCreate(role="user", content=message)]
        )

# Create once, use many times
helper = PersistentAgent()
response1 = helper.chat("Hello")
response2 = helper.chat("What did I just say?")  # Agent remembers!
```

### ❌ Overloading Core Memory

**Mistake:**
```python
# Wrong: Putting everything in one memory block
CreateBlock(
    label="everything",
    value="Customer: John Doe, Order: #12345, Issue: Billing, History: Called 3 times, Notes: Frustrated, Policy: 30-day returns, Account: Premium since 2020..."
)
```

**Problem:** Cluttered context, poor performance, hard to maintain.

**Solution:**
```python
# Right: Structured, purposeful memory blocks
memory_blocks = [
    CreateBlock(label="customer", value="John Doe - Premium member since 2020"),
    CreateBlock(label="current_issue", value="Billing inquiry for order #12345"),
    CreateBlock(label="context", value="Customer called 3 times, frustrated"),
    CreateBlock(label="agent_role", value="Billing support specialist"),
]

# Additional details go in archival memory for search
client.agents.memory.archival.insert(
    agent_id=agent.id,
    content="Customer history: 15 previous orders, no previous issues, always pays on time"
)
```

### ❌ Creating Tools for Everything

**Mistake:**
```python
# Wrong: Over-tooling with irrelevant capabilities
tools = [
    get_weather_tool,      # Not relevant for support agent
    play_music_tool,       # Not relevant for support agent
    send_tweet_tool,       # Not relevant for support agent
    calculate_tip_tool,    # Not relevant for support agent
    lookup_order_tool,     # Relevant
    process_refund_tool,   # Relevant
]
```

**Problem:** Confuses the agent, reduces performance, increases errors.

**Solution:**
```python
# Right: Focused, purpose-driven tool selection
support_tools = [
    lookup_order_tool,
    process_refund_tool,
    escalate_to_human_tool,
    send_notification_tool,
    update_customer_notes_tool,
]

# Create specialized agents for different domains
weather_agent = client.agents.create(tools=[get_weather_tool])
support_agent = client.agents.create(tools=support_tools)
```

## Memory Management Mistakes

### ❌ Not Understanding Memory Types

**Mistake:**
```python
# Wrong: Putting searchable content in core memory
CreateBlock(
    label="knowledge",
    value="All 500 pages of our product manual: Page 1: Introduction..."
)
```

**Problem:** Wastes context window, poor search capability.

**Solution:**
```python
# Right: Use appropriate memory types
# Core memory: Essential context
CreateBlock(label="role", value="Product support specialist")

# Archival memory: Searchable knowledge
file = client.files.upload_file(
    file_path="product_manual.pdf",
    description="Complete product documentation"
)
client.agents.files.attach(agent_id=agent.id, file_id=file.id)

# Message history: Conversation flow (automatic)
```

### ❌ Ignoring Context Window Limits

**Mistake:**
```python
# Wrong: Not monitoring context usage
while True:
    user_input = input("You: ")
    response = client.agents.messages.create(...)  # Eventually will fail
    print(response.messages[-1].content)
```

**Problem:** Agent stops working when context window fills up.

**Solution:**
```python
# Right: Monitor and handle context gracefully
def safe_chat(agent_id, message):
    try:
        response = client.agents.messages.create(
            agent_id=agent_id,
            messages=[MessageCreate(role="user", content=message)]
        )
        return response
    except ApiError as e:
        if "context_length" in str(e).lower():
            print("Context window full - agent will summarize automatically")
            # Letta handles this automatically, but you can monitor
            memory = client.agents.memory.retrieve(agent_id)
            print(f"Messages in memory: {len(memory.messages)}")
        raise
```

### ❌ Manual Memory Management Without Understanding

**Mistake:**
```python
# Wrong: Manually deleting memory without understanding impact
messages = client.agents.messages.list(agent_id)
for msg in messages[:-10]:  # Keep only last 10 messages
    client.agents.messages.delete(msg.id)  # This doesn't exist!
```

**Problem:** You can't manually delete individual messages; this breaks conversation flow.

**Solution:**
```python
# Right: Let Letta handle memory management automatically
# Or work with the memory system properly
memory = client.agents.memory.retrieve(agent_id)
print(f"Current message count: {len(memory.messages)}")

# If you need to reset memory, create a new agent or use archival
if len(memory.messages) > 1000:
    # Archive important information
    client.agents.memory.archival.insert(
        agent_id=agent_id,
        content="Summary of previous conversation: ..."
    )
    
    # Create fresh agent with same configuration
    # (Letta handles summarization automatically, this is rarely needed)
```

## Tool Development Mistakes

### ❌ Insecure Tool Functions

**Mistake:**
```python
# Wrong: Dangerous, unsecured tool
def execute_system_command(command: str) -> str:
    """Execute any system command."""
    import subprocess
    result = subprocess.run(command, shell=True, capture_output=True)
    return result.stdout.decode()

# This gives agent unlimited system access!
dangerous_tool = client.tools.upsert_from_function(func=execute_system_command)
```

**Problem:** Security vulnerability, agent could harm system.

**Solution:**
```python
# Right: Secure, limited-scope tools
def check_server_status(service_name: str) -> str:
    """Check if a specific service is running."""
    allowed_services = ["nginx", "apache", "postgres"]
    if service_name not in allowed_services:
        return f"Error: Service {service_name} not monitored"
    
    import subprocess
    try:
        result = subprocess.run(
            ["systemctl", "is-active", service_name],
            capture_output=True,
            text=True,
            timeout=5
        )
        return f"Service {service_name}: {result.stdout.strip()}"
    except subprocess.TimeoutExpired:
        return f"Timeout checking {service_name}"

# Secure tool with limited scope
status_tool = client.tools.upsert_from_function(func=check_server_status)
```

### ❌ Poor Tool Documentation

**Mistake:**
```python
# Wrong: Vague, unhelpful docstring
def do_stuff(data):
    """Does stuff with data."""
    # Agent has no idea what this does or how to use it
    return process_data(data)
```

**Problem:** Agent doesn't know when or how to use the tool.

**Solution:**
```python
# Right: Clear, detailed documentation
def calculate_shipping_cost(weight_kg: float, destination_country: str, service_type: str = "standard") -> str:
    """
    Calculate shipping cost for a package.
    
    Args:
        weight_kg: Package weight in kilograms (0.1 to 50.0)
        destination_country: ISO country code (e.g., "US", "CA", "GB")
        service_type: Shipping service - "standard", "express", or "overnight"
    
    Returns:
        Shipping cost in USD as a formatted string (e.g., "$15.99")
        
    Example:
        calculate_shipping_cost(2.5, "US", "express") -> "$24.99"
    """
    # Implementation...
    return f"${cost:.2f}"
```

### ❌ Tools That Return Too Much Data

**Mistake:**
```python
# Wrong: Returning massive datasets
def get_all_customers() -> str:
    """Get all customers from database."""
    customers = database.query("SELECT * FROM customers")  # Could be millions
    return json.dumps(customers)  # Overwhelms context window
```

**Problem:** Fills context window, poor performance, unusable output.

**Solution:**
```python
# Right: Paginated, filtered results
def search_customers(query: str, limit: int = 10) -> str:
    """
    Search customers by name or email.
    
    Args:
        query: Search term for customer name or email
        limit: Maximum number of results (1-50, default 10)
    
    Returns:
        JSON list of matching customers with basic info
    """
    limit = min(max(limit, 1), 50)  # Enforce reasonable limits
    customers = database.search_customers(query, limit=limit)
    
    # Return only essential information
    results = [
        {
            "id": c.id,
            "name": c.name,
            "email": c.email,
            "status": c.status
        }
        for c in customers
    ]
    
    return json.dumps(results, indent=2)
```

## API Usage Mistakes

### ❌ Not Handling Errors Properly

**Mistake:**
```python
# Wrong: Ignoring potential errors
def send_message_to_agent(agent_id, content):
    response = client.agents.messages.create(
        agent_id=agent_id,
        messages=[MessageCreate(role="user", content=content)]
    )
    return response.messages[-1].content  # Could crash
```

**Problem:** Brittle code that fails in production.

**Solution:**
```python
# Right: Comprehensive error handling
def send_message_to_agent(agent_id, content):
    try:
        response = client.agents.messages.create(
            agent_id=agent_id,
            messages=[MessageCreate(role="user", content=content)]
        )
        
        if not response.messages:
            return "Error: No response from agent"
            
        return response.messages[-1].content
        
    except ApiError as e:
        if e.status_code == 404:
            return f"Error: Agent {agent_id} not found"
        elif e.status_code == 429:
            return "Error: Rate limit exceeded, try again later"
        elif e.status_code >= 500:
            return "Error: Server error, please try again"
        else:
            return f"Error: {e.message}"
    except Exception as e:
        return f"Unexpected error: {str(e)}"
```

### ❌ Creating Agents in Loops

**Mistake:**
```python
# Wrong: Creating agents unnecessarily
def process_customer_requests(requests):
    responses = []
    for request in requests:
        # Creates new agent for each request!
        agent = client.agents.create(...)
        response = client.agents.messages.create(...)
        responses.append(response)
        client.agents.delete(agent.id)
    return responses
```

**Problem:** Slow, expensive, loses learning benefits.

**Solution:**
```python
# Right: Reuse agents appropriately
def process_customer_requests(requests):
    # Create one agent for all requests of the same type
    support_agent = client.agents.create(
        name="support-processor",
        memory_blocks=[...],
        tools=[...],
    )
    
    responses = []
    for request in requests:
        response = client.agents.messages.create(
            agent_id=support_agent.id,
            messages=[MessageCreate(role="user", content=request)]
        )
        responses.append(response)
        
        # Reset context between customers if needed
        client.agents.memory.update_block(
            agent_id=support_agent.id,
            block_id="current_customer",
            value=""  # Clear current customer context
        )
    
    return responses
```

## Configuration Mistakes

### ❌ Using Wrong Model for Task

**Mistake:**
```python
# Wrong: Using expensive model for simple tasks
simple_agent = client.agents.create(
    model="openai/gpt-4",  # Expensive for simple classification
    # Task: Just categorize support tickets
)
```

**Problem:** Unnecessary cost, slower responses.

**Solution:**
```python
# Right: Match model to task complexity
# Simple classification task
classifier_agent = client.agents.create(
    model="openai/gpt-4o-mini",  # Faster, cheaper
    memory_blocks=[
        CreateBlock(
            label="task", 
            value="Classify support tickets: billing, technical, or general"
        )
    ]
)

# Complex reasoning task
analysis_agent = client.agents.create(
    model="openai/gpt-4o",  # More capable for complex tasks
    memory_blocks=[
        CreateBlock(
            label="task",
            value="Analyze customer sentiment and provide detailed recommendations"
        )
    ]
)
```

### ❌ Ignoring Environment Differences

**Mistake:**
```python
# Wrong: Hardcoded configuration
client = Letta(base_url="http://localhost:8283")  # Only works locally
```

**Problem:** Breaks in different environments.

**Solution:**
```python
# Right: Environment-aware configuration
import os

def create_client():
    base_url = os.getenv("LETTA_SERVER_URL", "http://localhost:8283")
    token = os.getenv("LETTA_API_TOKEN")
    
    return Letta(
        base_url=base_url,
        token=token,
        timeout=int(os.getenv("LETTA_TIMEOUT", "60"))
    )

client = create_client()
```

### ❌ Not Validating Agent Configuration

**Mistake:**
```python
# Wrong: No validation of configuration
def create_agent_from_config(config):
    return client.agents.create(**config)  # Could fail with bad config
```

**Problem:** Runtime failures with unclear error messages.

**Solution:**
```python
# Right: Validate configuration before creation
def create_agent_from_config(config):
    # Validate required fields
    required_fields = ["model", "embedding"]
    for field in required_fields:
        if field not in config:
            raise ValueError(f"Missing required field: {field}")
    
    # Validate model availability
    available_models = [m.handle for m in client.models.list_llms()]
    if config["model"] not in available_models:
        raise ValueError(f"Model {config['model']} not available. Available: {available_models}")
    
    # Validate memory blocks
    if "memory_blocks" in config:
        for block in config["memory_blocks"]:
            if not isinstance(block, CreateBlock):
                raise ValueError("memory_blocks must be CreateBlock instances")
    
    return client.agents.create(**config)
```

## Prevention Strategies

### Use Type Hints
```python
from typing import List, Optional
from letta_client import Letta, AgentState, MessageCreate

def create_support_agent(
    client: Letta,
    name: str,
    tools: List[str],
    knowledge_files: Optional[List[str]] = None
) -> AgentState:
    # Type hints catch errors early
    pass
```

### Implement Validation
```python
def validate_agent_health(agent_id: str) -> dict:
    """Check agent configuration and memory state."""
    agent = client.agents.retrieve(agent_id)
    memory = client.agents.memory.retrieve(agent_id)
    
    health = {
        "agent_exists": bool(agent),
        "memory_blocks": len(memory.blocks),
        "message_count": len(memory.messages),
        "tools_count": len(agent.tools),
        "issues": []
    }
    
    # Check for common issues
    if len(memory.messages) > 1000:
        health["issues"].append("High message count - may need cleanup")
    
    if not memory.blocks:
        health["issues"].append("No core memory blocks defined")
    
    return health
```

### Use Monitoring
```python
def monitor_agent_usage(agent_id: str):
    """Monitor agent for performance and errors."""
    try:
        response = client.agents.messages.create(
            agent_id=agent_id,
            messages=[MessageCreate(role="user", content="Health check")]
        )
        print(f"✅ Agent {agent_id} responding normally")
        print(f"Token usage: {response.usage.total_tokens}")
    except Exception as e:
        print(f"❌ Agent {agent_id} error: {e}")
```

## Next Steps

- [Troubleshooting Guide](troubleshooting.md) - Diagnostic procedures
- [Performance Pitfalls](performance-pitfalls.md) - Optimization issues  
- [Security Considerations](security-considerations.md) - Security best practices

## See Also

- [Best Practices](../03-patterns/composition-patterns.md) - Recommended patterns
- [Error Handling](../03-patterns/error-handling.md) - Robust error management
- [Testing Strategies](../04-integration/testing-setup.md) - Testing approaches