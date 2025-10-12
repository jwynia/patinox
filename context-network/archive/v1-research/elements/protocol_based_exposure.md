# Protocol-Based Agent and Tool Exposure

## Overview

This document defines how Patinox agents and tools are exposed through standard protocols, emphasizing interoperability through A2A (Agent-to-Agent), MCP (Model Context Protocol), and other open standards. The design ensures that all agent capabilities are accessible through well-defined protocols rather than proprietary APIs.

## Core Principle: Protocol-First Design

All agent and tool functionality must be exposable through standard protocols. This ensures:

1. **Interoperability**: Agents can communicate regardless of implementation
2. **Standardization**: Common patterns across different systems
3. **Evolution**: Protocols can evolve independently of implementations
4. **Federation**: Agents can form networks across organizational boundaries

## Protocol Hierarchy

### 1. Model Context Protocol (MCP)

MCP serves as the foundational protocol for tool exposure:

```rust
/// MCP server implementation for exposing tools
pub struct MCPServer {
    tools: HashMap<String, Box<dyn Tool>>,
    transport: MCPTransport,
}

impl MCPServer {
    pub async fn expose_tool<T: Tool + 'static>(&mut self, tool: T) {
        let descriptor = ToolDescriptor {
            name: tool.name(),
            description: tool.description(),
            parameters: tool.parameter_schema(),
            returns: tool.return_schema(),
        };
        
        self.tools.insert(tool.name(), Box::new(tool));
        self.transport.register_tool(descriptor).await;
    }
    
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        match request {
            MCPRequest::ListTools => {
                MCPResponse::ToolList(self.list_tools())
            }
            MCPRequest::ExecuteTool { name, params } => {
                self.execute_tool(name, params).await
            }
            MCPRequest::GetSchema { tool } => {
                MCPResponse::Schema(self.get_tool_schema(tool))
            }
        }
    }
}

/// Client for consuming MCP tools
pub struct MCPClient {
    transport: MCPTransport,
    discovered_tools: HashMap<String, ToolDescriptor>,
}

impl MCPClient {
    pub async fn discover_tools(&mut self, endpoint: &str) -> Result<Vec<ToolDescriptor>> {
        let response = self.transport.request(
            endpoint,
            MCPRequest::ListTools
        ).await?;
        
        if let MCPResponse::ToolList(tools) = response {
            for tool in &tools {
                self.discovered_tools.insert(tool.name.clone(), tool.clone());
            }
            Ok(tools)
        } else {
            Err(Error::InvalidResponse)
        }
    }
    
    pub async fn execute(&self, tool: &str, params: Value) -> Result<Value> {
        self.transport.request(
            tool,
            MCPRequest::ExecuteTool {
                name: tool.to_string(),
                params,
            }
        ).await
    }
}
```

### 2. Agent-to-Agent (A2A) Protocol

A2A enables direct agent communication and coordination:

```rust
/// A2A protocol for inter-agent communication
pub struct A2AProtocol {
    agent_id: AgentId,
    capabilities: Vec<Capability>,
    handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
}

/// Core A2A message types
#[derive(Serialize, Deserialize)]
pub enum A2AMessage {
    /// Discover what an agent can do
    DiscoverCapabilities {
        requester: AgentId,
    },
    
    /// Request agent to perform task
    TaskRequest {
        task: Task,
        context: Context,
        callback: Option<CallbackEndpoint>,
    },
    
    /// Delegate subtask to another agent
    Delegation {
        parent_task: TaskId,
        subtask: Task,
        constraints: Constraints,
    },
    
    /// Share information between agents
    InformationShare {
        info_type: InformationType,
        data: Value,
        relevance: RelevanceScore,
    },
    
    /// Coordinate multi-agent workflow
    Coordination {
        workflow: WorkflowDescriptor,
        role: AgentRole,
        synchronization: SyncPoint,
    },
    
    /// Request agent status
    StatusQuery {
        query_type: StatusQueryType,
    },
}

impl A2AProtocol {
    pub async fn handle_message(&self, msg: A2AMessage) -> A2AResponse {
        match msg {
            A2AMessage::DiscoverCapabilities { .. } => {
                A2AResponse::Capabilities(self.capabilities.clone())
            }
            
            A2AMessage::TaskRequest { task, context, callback } => {
                let result = self.execute_task(task, context).await;
                if let Some(cb) = callback {
                    self.send_callback(cb, result.clone()).await;
                }
                A2AResponse::TaskResult(result)
            }
            
            A2AMessage::Delegation { parent_task, subtask, constraints } => {
                self.handle_delegation(parent_task, subtask, constraints).await
            }
            
            // ... other message types
        }
    }
    
    /// Register this agent with a network
    pub async fn join_network(&self, network: &NetworkEndpoint) -> Result<()> {
        let registration = AgentRegistration {
            id: self.agent_id.clone(),
            capabilities: self.capabilities.clone(),
            endpoint: self.get_endpoint(),
            metadata: self.get_metadata(),
        };
        
        network.register_agent(registration).await
    }
}
```

### 3. Web API Protocol Layer

HTTP/REST and WebSocket exposure for web clients:

```rust
/// Web API server exposing agents and tools
pub struct WebAPIServer {
    agents: HashMap<AgentId, Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
    sessions: SessionManager,
}

impl WebAPIServer {
    /// REST API endpoints
    pub fn configure_routes(&self) -> Router {
        Router::new()
            // Agent endpoints
            .route("/agents", get(self.list_agents))
            .route("/agents/:id", get(self.get_agent))
            .route("/agents/:id/execute", post(self.execute_agent))
            
            // Tool endpoints
            .route("/tools", get(self.list_tools))
            .route("/tools/:name", get(self.get_tool))
            .route("/tools/:name/execute", post(self.execute_tool))
            
            // Session management
            .route("/sessions", post(self.create_session))
            .route("/sessions/:id", delete(self.close_session))
            
            // WebSocket for streaming
            .route("/ws", ws(self.websocket_handler))
    }
    
    /// WebSocket handler for real-time communication
    async fn websocket_handler(&self, ws: WebSocket) {
        let (tx, rx) = ws.split();
        let session = self.sessions.create_websocket_session(tx).await;
        
        while let Some(msg) = rx.next().await {
            match msg {
                Message::Text(text) => {
                    let request: WebSocketRequest = serde_json::from_str(&text)?;
                    self.handle_websocket_request(session, request).await;
                }
                Message::Close(_) => {
                    self.sessions.close(session).await;
                    break;
                }
                _ => {}
            }
        }
    }
}

/// OpenAPI schema generation
impl WebAPIServer {
    pub fn generate_openapi_spec(&self) -> OpenAPI {
        let mut spec = OpenAPI::new("Patinox Agent API", "1.0.0");
        
        // Generate schemas from agents and tools
        for (id, agent) in &self.agents {
            spec.add_path(
                format!("/agents/{}", id),
                self.agent_to_openapi_path(agent)
            );
        }
        
        for (name, tool) in &self.tools {
            spec.add_path(
                format!("/tools/{}", name),
                self.tool_to_openapi_path(tool)
            );
        }
        
        spec
    }
}
```

### 4. GraphQL Protocol Support

For complex querying and subscriptions:

```rust
/// GraphQL schema for agents and tools
pub struct GraphQLSchema {
    agents: HashMap<AgentId, Arc<dyn Agent>>,
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl GraphQLSchema {
    pub fn build_schema() -> Schema {
        Schema::build("Query", "Mutation", "Subscription")
            .register_type::<Agent>()
            .register_type::<Tool>()
            .register_type::<Task>()
            .register_type::<Result>()
            .finish()
    }
}

#[Object]
impl QueryRoot {
    /// List all available agents
    async fn agents(&self) -> Vec<AgentInfo> {
        self.agents.values()
            .map(|a| a.info())
            .collect()
    }
    
    /// Get specific agent
    async fn agent(&self, id: String) -> Option<AgentInfo> {
        self.agents.get(&id)
            .map(|a| a.info())
    }
    
    /// List all available tools
    async fn tools(&self) -> Vec<ToolInfo> {
        self.tools.values()
            .map(|t| t.info())
            .collect()
    }
}

#[Object]
impl MutationRoot {
    /// Execute agent task
    async fn execute_agent(
        &self,
        agent_id: String,
        task: TaskInput,
    ) -> Result<TaskResult> {
        let agent = self.agents.get(&agent_id)
            .ok_or(Error::AgentNotFound)?;
        
        agent.execute(task.into()).await
    }
    
    /// Execute tool
    async fn execute_tool(
        &self,
        tool_name: String,
        params: Json<Value>,
    ) -> Result<Value> {
        let tool = self.tools.get(&tool_name)
            .ok_or(Error::ToolNotFound)?;
        
        tool.execute(params.0).await
    }
}

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to agent events
    async fn agent_events(&self, agent_id: String) -> impl Stream<Item = AgentEvent> {
        self.event_bus.subscribe_agent(agent_id)
    }
    
    /// Subscribe to task progress
    async fn task_progress(&self, task_id: String) -> impl Stream<Item = Progress> {
        self.event_bus.subscribe_task(task_id)
    }
}
```

### 5. gRPC Protocol Support

For high-performance binary communication:

```rust
/// gRPC service definitions
pub mod proto {
    tonic::include_proto!("patinox");
}

use proto::{agent_service_server::AgentService, tool_service_server::ToolService};

#[derive(Debug)]
pub struct AgentGrpcService {
    agents: Arc<AgentRegistry>,
}

#[tonic::async_trait]
impl AgentService for AgentGrpcService {
    async fn list_agents(
        &self,
        request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let agents = self.agents.list().await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(ListAgentsResponse { agents }))
    }
    
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        let agent = self.agents.get(&req.agent_id).await
            .ok_or_else(|| Status::not_found("Agent not found"))?;
        
        let result = agent.execute(req.task).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(ExecuteResponse { result }))
    }
    
    type StreamExecuteStream = ReceiverStream<Result<StreamUpdate, Status>>;
    
    async fn stream_execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<Self::StreamExecuteStream>, Status> {
        let (tx, rx) = mpsc::channel(128);
        let req = request.into_inner();
        
        let agent = self.agents.get(&req.agent_id).await
            .ok_or_else(|| Status::not_found("Agent not found"))?;
        
        tokio::spawn(async move {
            let mut stream = agent.execute_stream(req.task).await;
            while let Some(update) = stream.next().await {
                if tx.send(Ok(update)).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

## Protocol Negotiation

Automatic protocol selection based on client capabilities:

```rust
pub struct ProtocolNegotiator {
    supported_protocols: Vec<Protocol>,
}

#[derive(Clone)]
pub enum Protocol {
    MCP { version: String },
    A2A { version: String },
    REST { version: String },
    GraphQL,
    GRPC,
    WebSocket,
}

impl ProtocolNegotiator {
    pub fn negotiate(&self, client_capabilities: &[String]) -> Protocol {
        // Prefer protocols in order of efficiency
        let preference_order = vec![
            Protocol::GRPC,
            Protocol::A2A { version: "1.0".into() },
            Protocol::MCP { version: "1.0".into() },
            Protocol::GraphQL,
            Protocol::REST { version: "v1".into() },
            Protocol::WebSocket,
        ];
        
        for protocol in preference_order {
            if self.client_supports(&protocol, client_capabilities) {
                return protocol;
            }
        }
        
        // Fallback to REST
        Protocol::REST { version: "v1".into() }
    }
}
```

## Service Discovery

Agents and tools announce themselves for discovery:

```rust
pub struct ServiceDiscovery {
    registry: Arc<ServiceRegistry>,
    announcer: ServiceAnnouncer,
}

pub struct ServiceAnnouncer {
    mdns: MdnsAnnouncer,
    consul: Option<ConsulClient>,
    etcd: Option<EtcdClient>,
}

impl ServiceAnnouncer {
    pub async fn announce(&self, service: ServiceDescription) -> Result<()> {
        // Announce via mDNS for local network
        self.mdns.announce(&service).await?;
        
        // Register with Consul if configured
        if let Some(consul) = &self.consul {
            consul.register_service(&service).await?;
        }
        
        // Register with etcd if configured
        if let Some(etcd) = &self.etcd {
            etcd.put_service(&service).await?;
        }
        
        Ok(())
    }
}

pub struct ServiceDescription {
    pub name: String,
    pub service_type: ServiceType,
    pub protocols: Vec<Protocol>,
    pub endpoints: Vec<Endpoint>,
    pub capabilities: Vec<Capability>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub enum ServiceType {
    Agent,
    Tool,
    Workflow,
    Coordinator,
}
```

## Protocol Adapters

Convert between different protocol representations:

```rust
pub trait ProtocolAdapter {
    type Input;
    type Output;
    
    fn adapt(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Adapt MCP to A2A
pub struct MCPToA2AAdapter;

impl ProtocolAdapter for MCPToA2AAdapter {
    type Input = MCPRequest;
    type Output = A2AMessage;
    
    fn adapt(&self, input: MCPRequest) -> Result<A2AMessage> {
        match input {
            MCPRequest::ExecuteTool { name, params } => {
                Ok(A2AMessage::TaskRequest {
                    task: Task::ToolExecution { name, params },
                    context: Context::default(),
                    callback: None,
                })
            }
            MCPRequest::ListTools => {
                Ok(A2AMessage::DiscoverCapabilities {
                    requester: AgentId::anonymous(),
                })
            }
            _ => Err(Error::UnsupportedConversion)
        }
    }
}
```

## Security Considerations

### Authentication and Authorization

```rust
pub struct ProtocolSecurity {
    auth_provider: Box<dyn AuthProvider>,
    authz_provider: Box<dyn AuthzProvider>,
}

impl ProtocolSecurity {
    pub async fn authenticate(&self, request: &Request) -> Result<Identity> {
        // Extract credentials based on protocol
        let credentials = match request.protocol {
            Protocol::REST { .. } => self.extract_bearer_token(request)?,
            Protocol::GRPC => self.extract_grpc_metadata(request)?,
            Protocol::A2A { .. } => self.extract_a2a_signature(request)?,
            _ => Credentials::Anonymous,
        };
        
        self.auth_provider.authenticate(credentials).await
    }
    
    pub async fn authorize(
        &self,
        identity: &Identity,
        resource: &Resource,
        action: &Action,
    ) -> Result<bool> {
        self.authz_provider.check_permission(identity, resource, action).await
    }
}
```

### Rate Limiting

```rust
pub struct ProtocolRateLimiter {
    limiters: HashMap<Protocol, Box<dyn RateLimiter>>,
}

impl ProtocolRateLimiter {
    pub async fn check_limit(
        &self,
        protocol: &Protocol,
        identity: &Identity,
    ) -> Result<()> {
        let limiter = self.limiters.get(protocol)
            .ok_or(Error::NoLimiterForProtocol)?;
        
        limiter.check(identity).await
    }
}
```

## Protocol Evolution

Support for protocol versioning and migration:

```rust
pub struct ProtocolEvolution {
    versions: HashMap<Protocol, Vec<Version>>,
    migrations: HashMap<(Protocol, Version, Version), Box<dyn Migration>>,
}

impl ProtocolEvolution {
    pub fn migrate_request(
        &self,
        request: Request,
        from_version: Version,
        to_version: Version,
    ) -> Result<Request> {
        let key = (request.protocol.clone(), from_version, to_version);
        
        if let Some(migration) = self.migrations.get(&key) {
            migration.migrate(request)
        } else {
            Err(Error::NoMigrationPath)
        }
    }
}
```

## Best Practices

1. **Protocol Abstraction**: Always abstract protocol details from business logic
2. **Capability Discovery**: Support dynamic capability discovery
3. **Graceful Degradation**: Fall back to simpler protocols when needed
4. **Version Negotiation**: Support multiple protocol versions simultaneously
5. **Standard Compliance**: Follow protocol specifications exactly

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/dependency_injection_philosophy.md] - enables - Protocol implementations
  - [elements/configuration_strategy.md] - configures - Protocol settings
  - [foundation/principles.md] - follows - Interoperability principles

## Navigation Guidance
- **Access Context:** Reference when implementing agent/tool exposure
- **Common Next Steps:** Review workflow abstraction or CLI exposure
- **Related Tasks:** API design, protocol implementation, integration
- **Update Patterns:** Update when new protocols are supported

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial protocol-based exposure design with MCP, A2A, and web protocols