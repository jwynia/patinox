# Glossary

> Alphabetical reference of all Letta terms and concepts

## A

**ADE (Agent Development Environment)**  
Web-based interface for creating, testing, and observing Letta agents. Accessible at https://app.letta.com.

**Agent**  
A persistent, autonomous entity that processes messages, maintains memory, and executes tools. Retains context across conversations.

**Agent Group**  
Collection of agents working together using coordination strategies like round-robin or supervisor-worker patterns.

**Agent State**  
Current configuration and memory of an agent, including LLM settings, tools, and memory blocks.

**Alembic**  
Database migration tool used for managing schema changes in Letta's PostgreSQL database.

**API (Application Programming Interface)**  
REST endpoints provided by Letta server for programmatic access to agent functionality.

**Archival Memory**  
Long-term storage using vector embeddings for semantic search. Information stored as searchable passages.

**Authentication**  
Process of verifying user identity when accessing Letta server resources.

**Authorization**  
Checking whether an authenticated user has permission to perform specific operations.

## B

**Batch Agent**  
Specialized agent optimized for processing multiple requests in batch mode rather than interactive conversations.

**Block**  
Labeled unit of core memory containing specific information (e.g., `Block(label="human", value="Name: Sarah")`).

**Built-in Tools**  
Core functionality available to all agents by default, such as `send_message`, `core_memory_append`, and `archival_memory_search`.

## C

**Chunking**  
Process of breaking large documents into smaller, searchable pieces (passages) for embedding and retrieval.

**CLI (Command Line Interface)**  
Command-line tool (`letta`) for running agents and server operations locally.

**Client**  
Python SDK (`letta_client`) for interacting with Letta server programmatically.

**Composio**  
Platform for integrating Letta agents with third-party services and APIs like Gmail, Google Calendar, and Slack.

**Configuration**  
Settings that control agent behavior, including LLM provider, model selection, and memory limits.

**Content Extraction**  
Process of converting uploaded files (PDF, Word, etc.) into searchable text using tools like MarkItDown.

**Context Window**  
Maximum amount of text (in tokens) that can be processed by an LLM in a single request.

**Coordination Strategy**  
Method for managing interactions between multiple agents in a group (sequential, parallel, hierarchical).

**Core Memory**  
Essential, always-accessible information that's included in every LLM context. Like "working memory" or notes on your desk.

**CORS (Cross-Origin Resource Sharing)**  
Web security feature that allows Letta server to accept requests from different domains.

**Custom Tools**  
User-defined Python functions converted to agent tools via `client.tools.upsert_from_function()`.

## D

**Data Flow**  
How information moves through the Letta system from user input to agent response.

**Deployment**  
Process of running Letta in production environments, typically using Docker with PostgreSQL.

## E

**Embedding**  
Vector representation of text used for semantic search in archival memory. Enables finding information by meaning.

**Embedding Config**  
Configuration specifying which embedding model to use for generating vectors for archival memory search.

**Endpoint**  
Specific REST API URL for performing operations (e.g., `/v1/agents/{id}/messages`).

**Ephemeral Agent**  
Temporary agent for one-time tasks without persistent memory. Useful for testing or single-session operations.

**Eviction**  
Process of removing old information from memory to make space for new information.

## F

**File Upload**  
Process of adding documents (PDF, text, Word, etc.) to the system for agent access and search.

**Function Calling**  
LLM capability to request tool execution in structured format following OpenAI's function calling standard.

## G

**Group**  
See Agent Group.

## H

**Health Check**  
API endpoint (`/health`) for monitoring server status and connectivity.

**Human**  
Memory block label traditionally used for storing information about the user the agent is interacting with.

## I

**Integration**  
Connecting Letta with external systems, frameworks, or services.

**Invocation**  
The act of calling/executing a tool during agent processing.

## J

**JSON Schema**  
Format used for defining tool parameters and structured outputs in a way LLMs can understand.

## L

**Letta**  
The open source framework for building stateful agents with long-term memory (formerly MemGPT).

**Letta Agent**  
The standard agent implementation with full memory management and tool execution capabilities.

**LLM (Large Language Model)**  
AI model used for text generation and reasoning (e.g., GPT-4, Claude, Llama).

**LLM Config**  
Configuration specifying which language model to use, including provider, model name, and parameters.

## M

**MCP (Model Context Protocol)**  
Standard for connecting agents to external tools and data sources for interoperability.

**Memory**  
The persistent state system of an agent with multiple layers: core memory, archival memory, and message history.

**Memory Block**  
See Block.

**MemGPT**  
Previous name for Letta. Still appears in some legacy code and documentation.

**Message**  
Single unit of communication in a conversation, with role (user/assistant/system) and content.

**Message History**  
Chronological record of conversation between user and agent, with automatic summarization.

**Metadata**  
Additional information attached to passages for filtering and context (source file, date, author, etc.).

**Middleware**  
Code that processes HTTP requests before they reach handlers or after they return responses.

**Migration**  
Database schema change script managed by Alembic for evolving database structure.

## N

**Namespace**  
Logical grouping of resources for organization and access control.

## O

**ORM (Object-Relational Mapping)**  
Database layer using SQLAlchemy for data persistence, defining models like Agent, Message, Tool.

**Organization**  
Higher-level grouping of users and shared resources in enterprise deployments.

## P

**Passage**  
A chunk of information stored in archival memory with associated embeddings for semantic search.

**Persistence**  
Ability of agents to maintain state and memory across sessions and server restarts.

**Persona**  
Memory block traditionally used for storing agent personality and role information.

**Poetry**  
Python dependency management tool used for Letta development and installation.

**Pre-commit**  
Code quality tool that runs checks (formatting, linting) before git commits.

**Provider**  
LLM service provider (OpenAI, Anthropic, Ollama, etc.) with unified interface in Letta.

## Q

**Query**  
Search term used to find relevant information in archival memory or message history.

## R

**Recall**  
Process of retrieving information from archival memory based on semantic similarity.

**REST (Representational State Transfer)**  
Architectural style used for Letta's HTTP API endpoints.

**Retention**  
How long information stays in agent memory before being summarized or archived.

## S

**Sandbox**  
Isolated environment for safe tool execution, preventing tools from affecting the host system.

**Schema**  
Pydantic data validation models defining API request/response formats and tool specifications.

**SDK (Software Development Kit)**  
Client library (`letta_client`) for interacting with Letta programmatically.

**Semantic Search**  
Finding information based on meaning rather than exact keyword matches, using embeddings.

**Service Manager**  
Business logic layer components handling specific domains (AgentManager, MessageManager, etc.).

**Session**  
Extended conversation or interaction period with an agent.

**Shared Memory**  
Memory blocks or sources accessible to multiple agents in a group for team knowledge.

**Source**  
Knowledge base (documents, files, text) that agents can search and reference.

**State**  
Current memory and configuration of an agent at any point in time.

**Step**  
One iteration of agent processing: input → processing → output.

**Streaming**  
Real-time response generation where tokens are sent as they're generated for faster perceived response.

**Structured Output**  
LLM responses conforming to predefined JSON schemas for reliable data extraction.

**Summarization**  
Process of condensing older messages into summaries to manage context window limits.

## T

**Timeout**  
Maximum time allowed for tool execution before termination.

**Token**  
Unit of text processing for LLMs (roughly 3/4 of a word in English).

**Tool**  
Function that an agent can execute to interact with external systems or perform computations.

**Tool Execution**  
Process of running tools in sandboxed environments during agent processing.

**Tool Rules**  
Constraints controlling when and how agents use tools (terminal rules, init rules).

**Tool Schema**  
OpenAI-format specification describing tool parameters and behavior.

**TTL (Time To Live)**  
How long cached data remains valid before expiration.

**Turn**  
Complete user-agent interaction cycle (user message + agent response).

## U

**User**  
Actor in the system who owns agents, tools, and other resources. Enables multi-tenancy.

**UUID (Universally Unique Identifier)**  
Unique identifier format used for agents, tools, and other resources.

## V

**Vector Search**  
Finding relevant information using embedding similarity algorithms like cosine similarity.

**Voice Agent**  
Specialized agent for voice interactions with speech-to-text and text-to-speech capabilities.

## W

**Workspace**  
Conceptual "working space" where an agent assembles context for LLM processing.

## X, Y, Z

**YAML**  
Configuration file format used for some Letta settings and tool definitions.

---

## Quick Reference by Category

### **Agent Types**
- LettaAgent, VoiceAgent, EphemeralAgent, BatchAgent

### **Memory Types**  
- Core Memory, Archival Memory, Message History

### **Tool Types**
- Built-in Tools, Custom Tools, External Tools (MCP, Composio)

### **Configuration**
- LLM Config, Embedding Config, Tool Rules

### **Storage**
- Block, Passage, Source, File Upload

### **Infrastructure**
- Server, ADE, Client, API, ORM

### **Development**
- Poetry, Alembic, Pre-commit, Schema, Migration

## See Also

- [Terminology](terminology.md) - Detailed explanations of key terms
- [Architecture Overview](architecture-overview.md) - How components work together
- [API Reference](../02-api-reference/index.md) - Technical documentation