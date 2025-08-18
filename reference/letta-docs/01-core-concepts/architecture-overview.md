# Architecture Overview

> How Letta is organized and its main components

## System Architecture

Letta follows a **layered service architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│           Entry Layer                   │
│  ┌─────────────┐  ┌─────────────────┐  │
│  │ CLI (letta) │  │ REST API Server │  │
│  └─────────────┘  └─────────────────┘  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│           Server Layer                  │
│  ┌─────────────────────────────────────┐ │
│  │    Request Orchestration            │ │
│  │    Route Handling                   │ │
│  └─────────────────────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│          Service Layer                  │
│  ┌──────────────┐ ┌─────────────────┐  │
│  │ AgentManager │ │ MessageManager  │  │
│  │ ToolManager  │ │ BlockManager    │  │
│  │ FileManager  │ │ SourceManager   │  │
│  └──────────────┘ └─────────────────┘  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│           Agent Layer                   │
│  ┌─────────────────────────────────────┐ │
│  │  BaseAgent → LettaAgent            │ │
│  │  ├── VoiceAgent                    │ │
│  │  ├── EphemeralAgent                │ │
│  │  └── BatchAgent                    │ │
│  └─────────────────────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│          Data Layer                     │
│  ┌──────────────┐ ┌─────────────────┐  │
│  │ ORM Models   │ │ Pydantic Schema │  │
│  │ (Database)   │ │ (Validation)    │  │
│  └──────────────┘ └─────────────────┘  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│        Integration Layer                │
│  ┌──────────────┐ ┌─────────────────┐  │
│  │  LLM APIs    │ │   Tool System   │  │
│  │  (OpenAI,    │ │   (Functions,   │  │
│  │  Anthropic)  │ │   Sandboxes)    │  │
│  └──────────────┘ └─────────────────┘  │
└─────────────────────────────────────────┘
```

## Core Components

### 1. Entry Points
- **CLI** (`letta/cli/`): Command-line interface for development
- **REST API** (`letta/server/rest_api/`): Production HTTP endpoints
- **Client SDKs**: Python client (`letta_client`) for external integrations

### 2. Service Managers
Business logic encapsulated in manager classes:

- **AgentManager**: Agent lifecycle, state management
- **MessageManager**: Message persistence, retrieval, search
- **BlockManager**: Memory block operations
- **ToolManager**: Tool registration, validation, execution
- **FileManager**: File upload, processing, attachment
- **SourceManager**: Knowledge source management

### 3. Agent System
- **BaseAgent**: Abstract interface defining agent contract
- **LettaAgent**: Main production agent with memory management
- **Specialized Agents**: Voice, Ephemeral, Batch processing variants

### 4. Memory Architecture
- **Core Memory**: Essential agent information (persona, human context)
- **Archival Memory**: Long-term searchable storage using embeddings
- **Message History**: Conversation context with automatic summarization
- **Block System**: Structured memory with labels and content

### 5. Tool System
- **Function Registry**: Python functions converted to LLM tools
- **Execution Sandboxes**: Safe environments for tool execution
- **Schema Generation**: Automatic OpenAI tool schema creation
- **MCP Integration**: Model Context Protocol for external tools

## Architectural Patterns

### Actor Pattern
All operations require a `User` context for:
- **Permission Control**: Multi-tenant access control
- **Resource Isolation**: Users can only access their resources
- **Audit Logging**: Track actions by user

```python
# Every operation includes user context
agent_manager.create_agent(
    user=current_user,  # Actor pattern in action
    agent_config=config
)
```

### Factory Pattern
Dynamic creation of components:

```python
# LLM clients created via factory
llm_client = LLMClientFactory.create(
    provider="openai",
    model="gpt-4o-mini"
)

# Tools created from functions
tool = ToolFactory.create_from_function(my_function)
```

### Repository Pattern
Data access abstracted through service layer:

```python
# Never access ORM directly
# ❌ session.query(Agent).filter(...)

# ✅ Use service managers
agent = agent_manager.get_agent_by_id(agent_id, user=user)
```

### Strategy Pattern
Pluggable implementations:

```python
# Different agent types
agent = LettaAgent(...)      # Standard agent
agent = VoiceAgent(...)      # Voice-enabled agent
agent = EphemeralAgent(...)  # Temporary agent

# Different LLM providers
client = OpenAIClient(...)
client = AnthropicClient(...)
client = OllamaClient(...)
```

## Request Flow

### Typical Request Lifecycle

1. **Entry**: Request arrives via CLI or REST API
2. **Authentication**: User validation and context setup
3. **Routing**: Request routed to appropriate service manager
4. **Business Logic**: Service manager handles request logic
5. **Agent Processing**: Agent processes request through `step()` method
6. **LLM Communication**: LLMClient handles provider communication
7. **Tool Execution**: Tools executed in sandboxed environment
8. **State Persistence**: Changes saved via ORM layer
9. **Response**: Result returned through interface

### Example: Sending Message to Agent

```python
# 1. REST API receives request
POST /v1/agents/{agent_id}/messages

# 2. Router dispatches to message handler
@router.post("/agents/{agent_id}/messages")
async def create_message(agent_id: str, request: MessageCreate):

# 3. Service manager handles business logic
    message_manager.create_user_message(
        agent_id=agent_id,
        user=current_user,
        content=request.content
    )

# 4. Agent processes message
    agent = agent_manager.get_agent(agent_id, user=current_user)
    response = await agent.step(user_message)

# 5. LLM API call
    llm_response = await llm_client.chat_completion(messages)

# 6. Tool execution (if needed)
    if llm_response.tool_calls:
        tool_results = tool_executor.execute(tool_calls)

# 7. State persistence
    message_manager.save_messages(agent_messages)
    agent_manager.update_agent_state(agent)

# 8. Response returned
    return LettaResponse(messages=agent_messages)
```

## Data Flow

### Message Processing Flow

```
User Input → MessageManager → Agent → LLMClient → Tool Execution
    ↓             ↓              ↓         ↓            ↓
Database ←── State Updates ←── Memory ←── Response ←── Results
```

### Memory Management Flow

```
Message → Context Window → Summarization → Archival Storage
    ↓          Check           ↓              ↓
Store in ←─ Fits? ──No──→ Summarize ──→ Store Summary
Database      ↓Yes              ↓           Store Details
             Keep           Update Core      in Embeddings
```

## Directory Structure Mapping

```
letta/
├── main.py                 # Entry point (CLI)
├── server/                 # Server layer
│   ├── server.py          # Core server implementation
│   └── rest_api/          # REST API endpoints
├── services/              # Service layer (managers)
│   ├── agent_manager.py   # Agent lifecycle
│   ├── message_manager.py # Message handling
│   └── tool_manager.py    # Tool management
├── agents/                # Agent layer
│   ├── base_agent.py      # Abstract base
│   └── letta_agent.py     # Main implementation
├── schemas/               # Data validation layer
├── orm/                   # Database layer
├── llm_api/              # LLM integration layer
└── functions/            # Tool system
```

## Configuration Management

### Settings Hierarchy

1. **Environment Variables**: `LETTA_*` environment variables
2. **Config Files**: `~/.letta/config.json`
3. **Runtime Settings**: Passed via constructor parameters
4. **Defaults**: Built-in fallback values

### Key Configuration Areas

- **Database**: Connection strings, pool sizes
- **LLM Providers**: API keys, endpoints, model defaults
- **Server**: Port, CORS settings, authentication
- **Agent Defaults**: Memory limits, summarization settings
- **Tool Execution**: Sandbox configuration, timeout settings

## Scalability Considerations

### Horizontal Scaling
- **Stateless Services**: Service managers don't hold state
- **Database Sharding**: Agents can be distributed across databases
- **Load Balancing**: Multiple server instances behind load balancer

### Performance Optimizations
- **Connection Pooling**: Database and HTTP connection reuse
- **Async Operations**: Non-blocking I/O throughout the stack
- **Caching**: Memory and database query optimization
- **Lazy Loading**: On-demand resource initialization

## Next Steps

- [Key Abstractions](key-abstractions.md) - Main entities and relationships
- [Data Flow](data-flow.md) - How information moves through the system
- [Mental Model](mental-model.md) - The philosophy behind the design

## See Also

- [Service Manager Documentation](../02-api-reference/services/index.md)
- [Agent Implementation](../02-api-reference/agents/index.md)
- [🔗 Official Architecture Guide](https://docs.letta.com/concepts/architecture)