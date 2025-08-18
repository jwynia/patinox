# Terminology

> Domain-specific terms and their meanings in the Letta ecosystem

## Core Terms

### Agent
A persistent, autonomous entity that processes messages, maintains memory, and executes tools. Unlike stateless chatbots, agents retain context and improve over time.

**Synonyms**: LLM Agent, AI Assistant, Bot  
**Context**: "Create an agent for customer support"

### Memory
The persistent state system of an agent, consisting of multiple layers:
- **Core Memory**: Essential, always-accessible information
- **Archival Memory**: Long-term, searchable storage using embeddings
- **Message History**: Chronological conversation context

**Related**: Memory Block, Passage, Context Window

### Tool
A function that an agent can execute to interact with external systems or perform computations. Tools extend agent capabilities beyond text generation.

**Synonyms**: Function, Capability, Skill  
**Types**: Built-in tools, Custom tools, External tools

### Block
A labeled unit of core memory containing specific information (e.g., human info, persona, context).

**Format**: `Block(label="human", value="Name: Sarah, Role: Customer")`

### Passage
A chunk of information stored in archival memory with associated embeddings for semantic search.

**Context**: Retrieved during archival memory searches

### Source
A knowledge base (documents, files, text) that agents can search and reference.

**Examples**: PDF files, company documentation, web content

## Memory-Related Terms

### Core Memory
The "working memory" of an agent - essential information that's always included in the LLM context.

**Analogy**: Like keeping important notes on your desk

### Archival Memory
Long-term storage using vector embeddings for semantic search. Information is stored as passages.

**Analogy**: Like a searchable filing cabinet

### Message History
Chronological record of conversation between user and agent, with automatic summarization when context window fills.

**Management**: Sliding window with summarization

### Context Window
The maximum amount of text (in tokens) that can be processed by the LLM in a single request.

**Challenge**: Managing information flow when window becomes full

### Summarization
The process of condensing older messages into summaries to maintain context while respecting token limits.

**Modes**: Static buffer, Partial eviction

### Embedding
Vector representation of text used for semantic search in archival memory.

**Purpose**: Enables finding relevant information based on meaning, not just keywords

## Agent Types

### LettaAgent
The standard agent implementation with full memory management and tool execution capabilities.

**Use Case**: General-purpose conversational agents

### VoiceAgent
Agent specialized for voice interactions with audio processing capabilities.

**Features**: Speech-to-text, text-to-speech integration

### EphemeralAgent
Temporary agent for one-time tasks without persistent memory.

**Use Case**: Single-session operations, testing

### BatchAgent
Agent optimized for processing multiple requests in batch mode.

**Use Case**: Bulk operations, data processing

## Tool-Related Terms

### Built-in Tools
Core functionality available to all agents by default.

**Examples**: `send_message`, `core_memory_append`, `archival_memory_search`

### Custom Tools
User-defined Python functions converted to agent tools.

**Creation**: Via `client.tools.upsert_from_function()`

### Tool Schema
OpenAI-format specification describing tool parameters and behavior.

**Auto-generation**: Created automatically from Python function signatures

### Tool Execution
The process of running tools in sandboxed environments for security.

**Environments**: Local, E2B cloud, Modal serverless

### Tool Rules
Constraints controlling when and how agents use tools.

**Types**: Terminal rules (stop after use), Init rules (start with tool)

### MCP (Model Context Protocol)
Standard for connecting agents to external tools and data sources.

**Purpose**: Interoperability between different AI systems and tools

### Composio
Platform for integrating agents with third-party services and APIs.

**Examples**: Gmail, Google Calendar, Slack integrations

## Server and Infrastructure Terms

### Letta Server
The backend service that hosts agents and provides REST API endpoints.

**Access**: Via REST API, Python client, web interface

### ADE (Agent Development Environment)
Web-based interface for creating, testing, and observing agents.

**URL**: https://app.letta.com

### User
Actor in the system who owns agents, tools, and other resources. Enables multi-tenancy.

**Isolation**: Users can only access their own resources

### Organization
Higher-level grouping of users and shared resources in enterprise deployments.

**Purpose**: Team collaboration, resource sharing

### Service Manager
Business logic layer components that handle specific domains.

**Examples**: AgentManager, MessageManager, ToolManager

### ORM (Object-Relational Mapping)
Database layer using SQLAlchemy for data persistence.

**Models**: Agent, Message, Tool, User, etc.

## LLM Integration Terms

### LLM Config
Configuration specifying which language model to use for an agent.

**Components**: Provider, model name, API endpoint, parameters

### Embedding Config
Configuration for the embedding model used for semantic search.

**Purpose**: Vector generation for archival memory

### Provider
LLM service provider (OpenAI, Anthropic, Ollama, etc.).

**Support**: Multiple providers with unified interface

### Streaming
Real-time response generation where tokens are sent as they're generated.

**Benefit**: Faster perceived response time

### Function Calling
LLM capability to request tool execution in structured format.

**Standard**: OpenAI function calling format

### Structured Output
LLM responses conforming to predefined JSON schemas.

**Use Case**: Reliable data extraction, API responses

## File and Data Terms

### File Upload
Process of adding documents to the system for agent access.

**Formats**: PDF, text, Word, PowerPoint, etc.

### Content Extraction
Converting uploaded files to searchable text.

**Tools**: MarkItDown, custom parsers

### Chunking
Breaking large documents into smaller, searchable pieces (passages).

**Strategy**: Size-based, semantic, hybrid approaches

### Vector Search
Finding relevant information using embedding similarity.

**Algorithm**: Cosine similarity, approximate nearest neighbors

### Metadata
Additional information attached to passages for filtering and context.

**Examples**: Source file, creation date, author

## Multi-Agent Terms

### Agent Group
Collection of agents working together on tasks.

**Patterns**: Round-robin, supervisor-worker, dynamic coordination

### Coordination Strategy
Method for managing interactions between multiple agents.

**Types**: Sequential, parallel, hierarchical

### Shared Memory
Memory blocks or sources accessible to multiple agents in a group.

**Use Case**: Team knowledge, common context

## Development Terms

### Poetry
Python dependency management tool used for Letta development.

**Commands**: `poetry install`, `poetry shell`

### Alembic
Database migration tool for schema changes.

**Commands**: `alembic upgrade head`, `alembic revision`

### Pre-commit
Code quality tool that runs checks before commits.

**Setup**: `poetry run pre-commit install`

### Schema
Pydantic data validation models defining API request/response formats.

**Location**: `letta/schemas/`

### Migration
Database schema change script managed by Alembic.

**Purpose**: Evolving database structure over time

## Common Abbreviations

- **ADE**: Agent Development Environment
- **LLM**: Large Language Model
- **MCP**: Model Context Protocol
- **ORM**: Object-Relational Mapping
- **REST**: Representational State Transfer
- **API**: Application Programming Interface
- **SDK**: Software Development Kit
- **CLI**: Command Line Interface
- **UUID**: Universally Unique Identifier
- **TTL**: Time To Live
- **CORS**: Cross-Origin Resource Sharing

## Context-Specific Terms

### In Agent Context
- **Step**: One iteration of agent processing (input → processing → output)
- **Turn**: Complete user-agent interaction cycle
- **Session**: Extended conversation with an agent
- **State**: Current memory and configuration of an agent

### In Memory Context
- **Recall**: Retrieving information from archival memory
- **Retention**: How long information stays in memory
- **Consolidation**: Moving information from message history to archival
- **Eviction**: Removing old information to make space

### In Tool Context
- **Invocation**: Calling a tool during agent processing
- **Sandbox**: Isolated environment for tool execution
- **Permission**: Authorization to use specific tools
- **Timeout**: Maximum time allowed for tool execution

### In Server Context
- **Endpoint**: REST API URL for specific operations
- **Middleware**: Code that processes requests before/after handlers
- **Authentication**: Verifying user identity
- **Authorization**: Checking user permissions

## Deprecated Terms

### MemGPT
Previous name for Letta (still used in some contexts).

**Migration**: `memgpt` → `letta` in commands and imports

### Persona
Old term for part of core memory describing agent personality.

**Current**: Stored as memory block with label "persona"

### Human
Old term for part of core memory describing user information.

**Current**: Stored as memory block with label "human"

## Next Steps

- [Architecture Overview](architecture-overview.md) - Understanding system organization
- [Key Abstractions](key-abstractions.md) - Main entities and relationships
- [Glossary](glossary.md) - Alphabetical reference of all terms

## See Also

- [API Reference](../02-api-reference/index.md) - Technical documentation
- [Common Use Cases](../03-patterns/common-use-cases/index.md) - Terms in practice
- [🔗 Official Glossary](https://docs.letta.com/concepts/glossary)