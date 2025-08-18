# Import Patterns

> How to include and import Letta in various contexts

## Basic Import Patterns

### Client-Only Usage (Most Common)
```python
from letta_client import Letta, MessageCreate, CreateBlock
from letta_client.types import TerminalToolRule

# For creating agents and messaging
client = Letta(base_url="http://localhost:8283")
```

### Full Framework Access
```python
import letta
from letta import LettaAgent, AgentState, User
from letta.schemas.memory import Memory
from letta.schemas.tool import Tool

# For building custom agents and extending functionality
```

### Specific Component Imports
```python
# Memory components
from letta.schemas.block import Block
from letta.schemas.memory import BasicBlockMemory, ChatMemory

# Tool system
from letta.functions import Function
from letta.services.tool_manager import ToolManager

# LLM integrations
from letta.llm_api.openai_client import OpenAIClient
from letta.llm_api.anthropic_client import AnthropicClient
```

## Context-Specific Patterns

### Web Applications
```python
# FastAPI integration
from fastapi import FastAPI
from letta_client import Letta
import asyncio

app = FastAPI()
client = Letta(base_url="http://localhost:8283")

@app.post("/chat")
async def chat_endpoint(message: str, agent_id: str):
    response = client.agents.messages.create(
        agent_id=agent_id,
        messages=[MessageCreate(role="user", content=message)]
    )
    return {"response": response.messages[-1].content}
```

### Django Applications
```python
# Django views.py
from django.http import JsonResponse
from letta_client import Letta, MessageCreate
from django.conf import settings

class ChatView(View):
    def __init__(self):
        self.client = Letta(base_url=settings.LETTA_SERVER_URL)
    
    def post(self, request):
        # Handle chat logic
        pass
```

### Jupyter Notebooks
```python
# Cell 1: Setup
%load_ext autoreload
%autoreload 2

from letta_client import Letta, MessageCreate, CreateBlock
import os

# Cell 2: Configuration
client = Letta(base_url="http://localhost:8283")
os.environ["OPENAI_API_KEY"] = "your-key-here"

# Cell 3: Create and test agent
agent = client.agents.create(...)
```

### Command Line Tools
```python
#!/usr/bin/env python3
import sys
import argparse
from letta_client import Letta, MessageCreate

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--server", default="http://localhost:8283")
    parser.add_argument("--agent-id", required=True)
    parser.add_argument("message")
    
    args = parser.parse_args()
    client = Letta(base_url=args.server)
    
    response = client.agents.messages.create(
        agent_id=args.agent_id,
        messages=[MessageCreate(role="user", content=args.message)]
    )
    
    print(response.messages[-1].content)

if __name__ == "__main__":
    main()
```

## Environment-Specific Imports

### Server-Side Components
```python
# When building server extensions
from letta.server.rest_api.app import app
from letta.server.server import SyncServer
from letta.services.agent_manager import AgentManager
from letta.orm import User, Agent, Message
```

### Custom Agent Development
```python
# For advanced agent customization
from letta.agents.base_agent import BaseAgent
from letta.agents.letta_agent import LettaAgent
from letta.services.summarizer.summarizer import Summarizer
from letta.services.context_window_calculator import ContextWindowCalculator
```

### Tool Development
```python
# For creating custom tools
from letta.functions.functions import function
from letta.functions.schema_generator import generate_schema
from letta.services.tool_executor.tool_executor_base import ToolExecutorBase
```

## Package Structure Reference

```
letta/
├── __init__.py           # Main exports (AgentState, Memory, etc.)
├── agents/               # Agent implementations
├── client/               # Client utilities  
├── schemas/              # Data models (Pydantic)
├── services/             # Business logic managers
├── llm_api/              # LLM provider integrations
├── functions/            # Tool system
├── server/               # REST API server
└── orm/                  # Database models
```

## Import Optimization

### Lazy Imports for Large Applications
```python
# Avoid importing everything at startup
import importlib

def get_letta_client():
    letta_client = importlib.import_module('letta_client')
    return letta_client.Letta(base_url="http://localhost:8283")

# Use when needed
client = get_letta_client()
```

### Type Hints
```python
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from letta_client import Letta, AgentState
    from letta_client.types import MessageCreate

def create_agent(client: "Letta") -> "AgentState":
    # Implementation
    pass
```

## Configuration Patterns

### Environment-Based Configuration
```python
import os
from letta_client import Letta

def create_client():
    base_url = os.getenv("LETTA_SERVER_URL", "http://localhost:8283")
    token = os.getenv("LETTA_API_TOKEN")
    
    return Letta(
        base_url=base_url,
        token=token,
        timeout=int(os.getenv("LETTA_TIMEOUT", "60"))
    )
```

### Configuration Classes
```python
from dataclasses import dataclass
from letta_client import Letta

@dataclass
class LettaConfig:
    server_url: str = "http://localhost:8283"
    default_model: str = "openai/gpt-4o-mini"
    default_embedding: str = "openai/text-embedding-3-small"
    
    def create_client(self) -> Letta:
        return Letta(base_url=self.server_url)

config = LettaConfig()
client = config.create_client()
```

## Common Import Errors

### Missing Dependencies
```python
# ❌ Error: No module named 'letta_client'
# ✅ Solution: pip install letta

# ❌ Error: No module named 'letta.agents'  
# ✅ Solution: pip install -U letta (or check Python path)
```

### Version Compatibility
```python
# Check installed version
import letta
print(f"Letta version: {letta.__version__}")

# Ensure compatibility
if letta.__version__ < "0.11.0":
    raise RuntimeError("Letta 0.11.0+ required")
```

## Next Steps

- [Prerequisites](prerequisites.md) - Understanding what you need to know
- [API Reference](../02-api-reference/index.md) - Detailed import documentation
- [Architecture Overview](../01-core-concepts/architecture-overview.md) - Understanding the structure

## See Also

- [Client Configuration](../04-integration/build-configuration.md)
- [Server Setup](installation.md)
- [🔗 Python Package Index](https://pypi.org/project/letta/)