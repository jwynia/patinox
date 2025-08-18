# Tools API

> Create, manage, and execute custom agent tools

## Overview

The Tools API allows you to extend agent capabilities by creating custom functions that agents can execute. Tools enable agents to:

- Interact with external APIs and services
- Process data and perform calculations
- Execute business logic and workflows
- Access databases and file systems (in sandboxed environments)

## Quick Start

```python
from letta_client import Letta

client = Letta(base_url="http://localhost:8283")

# Create tool from function
def get_weather(location: str) -> str:
    """Get current weather for a location."""
    return f"Sunny and 72°F in {location}"

tool = client.tools.upsert_from_function(func=get_weather)

# Attach to agent
client.agents.tools.attach(agent_id="agent_123", tool_id=tool.id)

# Agent can now use the tool in conversations
```

## Tool Management

### Create Tool from Function

Convert a Python function into an agent tool with automatic schema generation.

```python
def upsert_from_function(
    self,
    func: callable,
    name: Optional[str] = None,
    description: Optional[str] = None,
    tags: Optional[List[str]] = None
) -> Tool:
    """Create or update a tool from a Python function.
    
    Args:
        func: Python function to convert to tool
        name: Custom tool name (defaults to function name)
        description: Custom description (defaults to docstring)
        tags: Tags for tool categorization
        
    Returns:
        Tool: Created tool with generated schema
        
    Raises:
        ApiError: If function invalid or name conflicts
        ValidationError: If function signature unsupported
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `func` | callable | **Yes** | N/A | Valid Python function | Function to convert |
| `name` | str | No | func.__name__ | 1-64 chars, alphanumeric + underscores | Tool identifier |
| `description` | str | No | func.__doc__ | 1-512 chars | Tool description |
| `tags` | List[str] | No | [] | Max 10 tags, 1-32 chars each | Categorization tags |

#### Function Requirements

**Supported Types:**
- `str`, `int`, `float`, `bool` - Basic types
- `List[T]`, `Dict[str, T]` - Collections (where T is supported type)
- `Optional[T]` - Optional parameters
- `Union[str, int]` - Simple unions

**Function Constraints:**
- Must have type hints for all parameters
- Must have return type hint
- Must have docstring describing functionality
- No complex objects or classes as parameters
- No async functions (use sync wrappers)

**Example:**
```python
def calculate_discount(
    price: float, 
    discount_percent: int, 
    customer_tier: str = "standard"
) -> str:
    """Calculate discounted price for a customer.
    
    Args:
        price: Original price in USD
        discount_percent: Discount percentage (0-100)
        customer_tier: Customer tier (standard, premium, vip)
        
    Returns:
        Formatted price string with discount applied
    """
    if customer_tier == "vip":
        discount_percent += 5
    elif customer_tier == "premium":
        discount_percent += 2
    
    final_price = price * (1 - discount_percent / 100)
    return f"${final_price:.2f} (was ${price:.2f})"

tool = client.tools.upsert_from_function(
    func=calculate_discount,
    tags=["pricing", "business"]
)
```

### List Tools

Get all tools available to the current user.

```python
def list(
    self,
    name: Optional[str] = None,
    tags: Optional[List[str]] = None,
    limit: int = 50,
    cursor: Optional[str] = None
) -> List[Tool]:
    """List tools available to the current user.
    
    Args:
        name: Filter by tool name (partial match)
        tags: Filter by tags (must have all specified tags)
        limit: Maximum number of tools to return
        cursor: Pagination cursor
        
    Returns:
        List[Tool]: Available tools
        
    Raises:
        ApiError: If query invalid
    """
```

#### Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `name` | str | No | None | 1-64 chars | Name filter (partial match) |
| `tags` | List[str] | No | None | Max 10 tags | Tag filter (AND logic) |
| `limit` | int | No | 50 | 1-200 | Max results |
| `cursor` | str | No | None | Valid cursor | Pagination token |

**Example:**
```python
# List all tools
all_tools = client.tools.list()

# Filter by name
weather_tools = client.tools.list(name="weather")

# Filter by tags
business_tools = client.tools.list(tags=["business", "pricing"])

# Pagination
first_page = client.tools.list(limit=10)
next_page = client.tools.list(limit=10, cursor=first_page[-1].id)
```

### Retrieve Tool

Get detailed information about a specific tool.

```python
def retrieve(
    self,
    tool_id: str
) -> Tool:
    """Retrieve detailed information about a tool.
    
    Args:
        tool_id: UUID of the tool
        
    Returns:
        Tool: Complete tool information
        
    Raises:
        ApiError: If tool not found or access denied
    """
```

**Example:**
```python
tool = client.tools.retrieve(tool_id="tool_123")
print(f"Tool: {tool.name}")
print(f"Description: {tool.description}")
print(f"Parameters: {tool.parameters}")
```

### Update Tool

Modify tool metadata (name, description, tags).

```python
def update(
    self,
    tool_id: str,
    name: Optional[str] = None,
    description: Optional[str] = None,
    tags: Optional[List[str]] = None
) -> Tool:
    """Update tool metadata.
    
    Args:
        tool_id: UUID of the tool to update
        name: New tool name
        description: New description
        tags: New tags list
        
    Returns:
        Tool: Updated tool
        
    Raises:
        ApiError: If tool not found or name conflicts
    """
```

**Note**: Cannot update function logic or parameters - create new tool instead.

### Delete Tool

Remove a tool and detach it from all agents.

```python
def delete(
    self,
    tool_id: str
) -> None:
    """Delete a tool and remove from all agents.
    
    Args:
        tool_id: UUID of the tool to delete
        
    Raises:
        ApiError: If tool not found or is built-in tool
    """
```

**Warning**: Cannot delete built-in tools (send_message, memory operations, etc.).

## Tool Schemas

### Tool Object

```python
class Tool:
    id: str                         # Tool UUID
    name: str                       # Tool identifier
    description: str                # Tool description
    parameters: dict                # OpenAI function schema
    tags: List[str]                 # Categorization tags
    created_at: datetime           # Creation timestamp
    updated_at: datetime           # Last modification
    user_id: str                   # Owner user ID
    
class ToolCall:
    id: str                        # Call UUID
    name: str                      # Tool name
    arguments: dict                # Call parameters
    
class ToolResult:
    tool_call_id: str              # Corresponding call ID
    output: Optional[str]          # Success output
    error: Optional[str]           # Error message if failed
```

### Generated Schema Example

For the discount calculation function above, Letta generates:

```json
{
  "type": "function",
  "function": {
    "name": "calculate_discount",
    "description": "Calculate discounted price for a customer.",
    "parameters": {
      "type": "object",
      "properties": {
        "price": {
          "type": "number",
          "description": "Original price in USD"
        },
        "discount_percent": {
          "type": "integer",
          "description": "Discount percentage (0-100)"
        },
        "customer_tier": {
          "type": "string",
          "description": "Customer tier (standard, premium, vip)",
          "default": "standard"
        }
      },
      "required": ["price", "discount_percent"]
    }
  }
}
```

## Agent Tool Operations

### Attach Tool to Agent

Add a tool to an agent's available tools.

```python
def agents.tools.attach(
    self,
    agent_id: str,
    tool_id: str
) -> None:
    """Attach a tool to an agent.
    
    Args:
        agent_id: UUID of the target agent
        tool_id: UUID of the tool to attach
        
    Raises:
        ApiError: If agent/tool not found
        ValidationError: If agent already has max tools
    """
```

#### Parameters

| Parameter | Type | Required | Constraints | Description |
|-----------|------|----------|-------------|-------------|
| `agent_id` | str | **Yes** | Valid UUID | Agent identifier |
| `tool_id` | str | **Yes** | Valid UUID | Tool identifier |

**Constraints:**
- Maximum 50 tools per agent
- Tool must be owned by same user as agent
- Cannot attach same tool twice

### Detach Tool from Agent

Remove a tool from an agent.

```python
def agents.tools.detach(
    self,
    agent_id: str,
    tool_id: str
) -> None:
    """Detach a tool from an agent.
    
    Args:
        agent_id: UUID of the target agent
        tool_id: UUID of the tool to detach
        
    Raises:
        ApiError: If agent/tool not found
        ValidationError: If trying to detach required tool
    """
```

**Note**: Cannot detach built-in tools that are required for agent operation.

### List Agent Tools

Get all tools available to a specific agent.

```python
def agents.tools.list(
    self,
    agent_id: str
) -> List[Tool]:
    """List all tools available to an agent.
    
    Args:
        agent_id: UUID of the target agent
        
    Returns:
        List[Tool]: Agent's available tools
        
    Raises:
        ApiError: If agent not found
    """
```

**Example:**
```python
agent_tools = client.agents.tools.list(agent_id="agent_123")
print(f"Agent has {len(agent_tools)} tools:")
for tool in agent_tools:
    print(f"- {tool.name}: {tool.description}")
```

## Built-in Tools

### Core Tools (Always Available)

| Tool Name | Description | Parameters |
|-----------|-------------|------------|
| `send_message` | Send message to user | content: str |
| `pause_heartbeats` | Pause agent processing | minutes: int |
| `core_memory_append` | Add to core memory block | label: str, content: str |
| `core_memory_replace` | Replace core memory block | label: str, content: str |
| `conversation_search` | Search message history | query: str, page: int |
| `conversation_search_date` | Search by date range | query: str, start_date: str, end_date: str |
| `archival_memory_insert` | Add to archival memory | content: str |
| `archival_memory_search` | Search archival memory | query: str, page: int |

### Optional Built-in Tools

| Tool Name | Description | Enable With |
|-----------|-------------|-------------|
| `web_search` | Search the internet | External tools package |
| `send_email` | Send email messages | Email integration |
| `file_search` | Search uploaded files | File processing |
| `database_query` | Query connected databases | Database integration |

## Tool Development Patterns

### API Integration Tool

```python
def fetch_user_profile(user_id: str) -> str:
    """Fetch user profile from external API.
    
    Args:
        user_id: User identifier to fetch
        
    Returns:
        JSON string containing user profile data
    """
    import requests
    
    try:
        response = requests.get(
            f"https://api.example.com/users/{user_id}",
            headers={"Authorization": f"Bearer {os.getenv('API_TOKEN')}"},
            timeout=10
        )
        response.raise_for_status()
        return response.json()
    except requests.RequestException as e:
        return f"Error fetching user profile: {str(e)}"

tool = client.tools.upsert_from_function(
    func=fetch_user_profile,
    tags=["api", "user-management"]
)
```

### Data Processing Tool

```python
def analyze_sales_data(data_csv: str, metric: str = "revenue") -> str:
    """Analyze sales data and return insights.
    
    Args:
        data_csv: CSV data as string
        metric: Metric to analyze (revenue, units, growth)
        
    Returns:
        Analysis summary with key insights
    """
    import pandas as pd
    from io import StringIO
    
    try:
        df = pd.read_csv(StringIO(data_csv))
        
        if metric == "revenue":
            total = df['revenue'].sum()
            avg = df['revenue'].mean()
            return f"Total revenue: ${total:,.2f}, Average: ${avg:,.2f}"
        elif metric == "units":
            total = df['units'].sum()
            return f"Total units sold: {total:,}"
        else:
            return f"Unsupported metric: {metric}"
            
    except Exception as e:
        return f"Error analyzing data: {str(e)}"
```

### Business Logic Tool

```python
def process_order(
    customer_id: str, 
    items: List[str], 
    discount_code: str = None
) -> str:
    """Process a customer order with business rules.
    
    Args:
        customer_id: Customer identifier
        items: List of item SKUs to order
        discount_code: Optional discount code
        
    Returns:
        Order confirmation with total and order ID
    """
    # Business logic implementation
    base_total = len(items) * 29.99  # Simplified pricing
    
    discount = 0
    if discount_code == "SAVE10":
        discount = base_total * 0.1
    elif discount_code == "VIP":
        discount = base_total * 0.15
    
    final_total = base_total - discount
    order_id = f"ORD-{hash(customer_id + str(items))}"[:10]
    
    return f"Order {order_id} confirmed for ${final_total:.2f} (saved ${discount:.2f})"
```

## Error Handling

### Tool Creation Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Invalid function signature | Missing type hints | Add type hints to all parameters |
| Unsupported types | Complex objects in signature | Use basic types (str, int, float, bool) |
| Missing docstring | No function description | Add descriptive docstring |
| Name conflict | Tool name already exists | Use different name or update existing |

```python
try:
    tool = client.tools.upsert_from_function(func=my_function)
except ApiError as e:
    if e.status_code == 422:
        if "type hints" in e.message.lower():
            print("Function needs type hints for all parameters")
        elif "docstring" in e.message.lower():
            print("Function needs a descriptive docstring")
        elif "unsupported type" in e.message.lower():
            print("Use basic types: str, int, float, bool, List, Dict")
```

### Tool Execution Errors

```python
# Tools should handle errors gracefully
def safe_api_call(endpoint: str) -> str:
    """Make API call with error handling."""
    try:
        response = requests.get(endpoint, timeout=5)
        response.raise_for_status()
        return response.text
    except requests.Timeout:
        return "Error: API request timed out"
    except requests.RequestException as e:
        return f"Error: API request failed - {str(e)}"
    except Exception as e:
        return f"Error: Unexpected error - {str(e)}"
```

## Security Considerations

### Sandboxed Execution

Tools run in isolated environments with restricted access:

- **Network**: Limited to allowed domains
- **File System**: Read-only access to specific directories
- **Resources**: CPU and memory limits
- **Time**: Execution timeouts

### Best Practices

```python
# Good: Safe, limited scope
def get_weather(city: str) -> str:
    """Get weather for a specific city."""
    allowed_cities = ["New York", "London", "Tokyo"]
    if city not in allowed_cities:
        return f"Weather data not available for {city}"
    
    # Safe API call
    return f"Sunny and 72°F in {city}"

# Bad: Dangerous, unlimited scope
def execute_command(command: str) -> str:
    """Execute any system command."""
    import subprocess
    # This would be blocked by sandbox
    result = subprocess.run(command, shell=True, capture_output=True)
    return result.stdout.decode()
```

### Input Validation

```python
def calculate_loan_payment(
    principal: float, 
    rate: float, 
    years: int
) -> str:
    """Calculate monthly loan payment."""
    # Validate inputs
    if principal <= 0 or principal > 10_000_000:
        return "Error: Principal must be between $1 and $10M"
    
    if rate < 0 or rate > 50:
        return "Error: Interest rate must be between 0% and 50%"
    
    if years <= 0 or years > 50:
        return "Error: Loan term must be between 1 and 50 years"
    
    # Safe calculation
    monthly_rate = rate / 100 / 12
    num_payments = years * 12
    
    if rate == 0:
        payment = principal / num_payments
    else:
        payment = principal * (monthly_rate * (1 + monthly_rate)**num_payments) / ((1 + monthly_rate)**num_payments - 1)
    
    return f"Monthly payment: ${payment:.2f}"
```

## Performance Tips

### Efficient Tool Design

```python
# Good: Fast, focused operation
def calculate_tax(amount: float, tax_rate: float) -> str:
    """Calculate tax amount."""
    tax = amount * tax_rate / 100
    return f"Tax: ${tax:.2f}, Total: ${amount + tax:.2f}"

# Bad: Slow, complex operation
def analyze_entire_database() -> str:
    """Analyze all data in database."""
    # This would timeout and consume too many resources
    pass
```

### Caching Expensive Operations

```python
from functools import lru_cache

@lru_cache(maxsize=100)
def get_exchange_rate(from_currency: str, to_currency: str) -> str:
    """Get exchange rate between currencies (cached)."""
    # Expensive API call cached for performance
    rate = fetch_rate_from_api(from_currency, to_currency)
    return f"1 {from_currency} = {rate} {to_currency}"
```

## Next Steps

- [Agents API](../agents/index.md) - Managing agent-tool relationships
- [Files API](../files/index.md) - File-based tools and knowledge
- [Tool Patterns](../../03-patterns/tool-patterns.md) - Advanced tool development

## See Also

- [Tool Concepts](../../01-core-concepts/key-abstractions.md#tools) - Understanding tools
- [Common Mistakes](../../05-gotchas/common-mistakes.md#tool-development-mistakes) - Tool pitfalls
- [Security Guide](../../05-gotchas/security-considerations.md) - Tool security