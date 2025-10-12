# WebSocket Hub Architecture

## Overview

WebSocket hubs provide the real-time, bidirectional communication infrastructure necessary for distributed agent conversations. Unlike traditional request-response patterns, WebSocket hubs maintain persistent connections that enable instant message broadcasting, event streaming, and dynamic participant management. This document details the architectural patterns for building scalable, resilient WebSocket hub systems.

## Core Architecture

### Hub Topology Patterns

Different topologies serve different scaling and reliability needs:

```rust
pub enum HubTopology {
    /// Single centralized hub
    Centralized {
        hub: Arc<CentralHub>,
        backup: Option<Arc<CentralHub>>,
    },
    
    /// Multiple hubs with coordination
    Federated {
        hubs: Vec<Arc<FederatedHub>>,
        coordinator: Arc<FederationCoordinator>,
    },
    
    /// Hierarchical hub structure
    Hierarchical {
        root: Arc<RootHub>,
        regional: HashMap<Region, Arc<RegionalHub>>,
        local: HashMap<Location, Arc<LocalHub>>,
    },
    
    /// Peer-to-peer mesh
    Mesh {
        nodes: Vec<Arc<MeshNode>>,
        routing_table: Arc<RoutingTable>,
    },
    
    /// Hybrid approach
    Hybrid {
        core_hubs: Vec<Arc<CoreHub>>,
        edge_nodes: Vec<Arc<EdgeNode>>,
        cdn_layer: Option<Arc<CDNLayer>>,
    },
}
```

### Central Hub Implementation

The fundamental hub pattern:

```rust
pub struct CentralHub {
    /// Unique hub identifier
    hub_id: HubId,
    
    /// Active WebSocket connections
    connections: Arc<RwLock<HashMap<ConnectionId, WebSocketConnection>>>,
    
    /// Participant registry
    participants: Arc<RwLock<HashMap<ParticipantId, ParticipantInfo>>>,
    
    /// Conversation rooms
    conversations: Arc<RwLock<HashMap<ConversationId, ConversationRoom>>>,
    
    /// Message router
    router: Arc<MessageRouter>,
    
    /// Event bus for internal events
    event_bus: Arc<EventBus>,
    
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

pub struct WebSocketConnection {
    /// Connection identifier
    id: ConnectionId,
    
    /// Associated participant
    participant_id: Option<ParticipantId>,
    
    /// WebSocket sender
    sender: Arc<Mutex<WebSocketSender>>,
    
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    
    /// Heartbeat manager
    heartbeat: HeartbeatManager,
    
    /// Message queue for buffering
    queue: Arc<Mutex<MessageQueue>>,
}

pub enum ConnectionState {
    /// Initial connection, not authenticated
    Connected {
        connected_at: Instant,
    },
    
    /// Authenticated and active
    Authenticated {
        participant: ParticipantId,
        permissions: Permissions,
    },
    
    /// Temporarily disconnected
    Suspended {
        since: Instant,
        resume_token: ResumeToken,
    },
    
    /// Closing connection
    Closing {
        reason: CloseReason,
    },
}

impl CentralHub {
    /// Accept new WebSocket connection
    pub async fn accept_connection(&self, ws: WebSocket) -> Result<()> {
        let connection_id = ConnectionId::new();
        let (sender, receiver) = ws.split();
        
        // Create connection wrapper
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            participant_id: None,
            sender: Arc::new(Mutex::new(sender)),
            state: Arc::new(RwLock::new(ConnectionState::Connected {
                connected_at: Instant::now(),
            })),
            heartbeat: HeartbeatManager::new(Duration::from_secs(30)),
            queue: Arc::new(Mutex::new(MessageQueue::new(1000))),
        };
        
        // Register connection
        self.connections.write().await.insert(connection_id.clone(), connection.clone());
        
        // Start connection handlers
        self.spawn_receiver_task(connection_id.clone(), receiver);
        self.spawn_sender_task(connection_id.clone(), connection.clone());
        self.spawn_heartbeat_task(connection_id.clone(), connection.clone());
        
        Ok(())
    }
    
    /// Route message to recipients
    pub async fn route_message(&self, message: Message) -> Result<()> {
        let routing_decision = self.router.route(&message).await?;
        
        match routing_decision {
            RoutingDecision::Broadcast(conversation_id) => {
                self.broadcast_to_conversation(conversation_id, message).await?;
            }
            
            RoutingDecision::Multicast(recipients) => {
                self.send_to_participants(recipients, message).await?;
            }
            
            RoutingDecision::Unicast(recipient) => {
                self.send_to_participant(recipient, message).await?;
            }
            
            RoutingDecision::Store => {
                self.store_message(message).await?;
            }
        }
        
        Ok(())
    }
}
```

### Message Routing and Broadcasting

Efficient message distribution patterns:

```rust
pub struct MessageRouter {
    /// Routing strategy
    strategy: RoutingStrategy,
    
    /// Message filters
    filters: Vec<Box<dyn MessageFilter>>,
    
    /// Routing cache for performance
    cache: Arc<RoutingCache>,
}

pub enum RoutingStrategy {
    /// Direct routing based on recipient
    Direct,
    
    /// Topic-based pub/sub
    TopicBased {
        subscriptions: Arc<RwLock<HashMap<Topic, HashSet<ParticipantId>>>>,
    },
    
    /// Content-based routing
    ContentBased {
        rules: Vec<RoutingRule>,
    },
    
    /// Intelligent routing with ML
    Intelligent {
        model: Arc<dyn RoutingModel>,
    },
}

pub struct BroadcastManager {
    /// Broadcast strategies
    strategy: BroadcastStrategy,
    
    /// Connection pools for efficiency
    pools: ConnectionPools,
}

pub enum BroadcastStrategy {
    /// Send to all immediately
    Immediate,
    
    /// Batch messages for efficiency
    Batched {
        window: Duration,
        max_batch_size: usize,
    },
    
    /// Priority-based delivery
    Prioritized {
        queues: PriorityQueues,
    },
    
    /// Multicast groups
    Multicast {
        groups: HashMap<GroupId, Vec<ConnectionId>>,
    },
}

impl BroadcastManager {
    pub async fn broadcast(
        &self,
        recipients: Vec<ConnectionId>,
        message: Message,
    ) -> Result<BroadcastResult> {
        match self.strategy {
            BroadcastStrategy::Immediate => {
                let futures = recipients.iter().map(|conn_id| {
                    self.send_to_connection(conn_id.clone(), message.clone())
                });
                
                let results = futures::future::join_all(futures).await;
                self.analyze_results(results)
            }
            
            BroadcastStrategy::Batched { window, max_batch_size } => {
                self.add_to_batch(recipients, message).await;
                
                if self.should_flush_batch(max_batch_size) {
                    self.flush_batch().await
                } else {
                    Ok(BroadcastResult::Queued)
                }
            }
            
            BroadcastStrategy::Multicast { ref groups } => {
                // Use multicast groups for efficiency
                let group_id = self.find_or_create_group(&recipients).await?;
                self.multicast_to_group(group_id, message).await
            }
            
            // ... other strategies
        }
    }
}
```

### Connection Management

Handling connection lifecycle and resilience:

```rust
pub struct ConnectionManager {
    /// Connection pool
    pool: ConnectionPool,
    
    /// Reconnection strategy
    reconnection: ReconnectionStrategy,
    
    /// Session management
    sessions: SessionManager,
}

pub struct ConnectionPool {
    /// Maximum connections per participant
    max_per_participant: usize,
    
    /// Total connection limit
    max_total: usize,
    
    /// Idle timeout
    idle_timeout: Duration,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<ConnectionId, PooledConnection>>>,
}

pub enum ReconnectionStrategy {
    /// Simple exponential backoff
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
    
    /// Smart reconnection with state preservation
    Stateful {
        resume_window: Duration,
        state_store: Arc<StateStore>,
    },
    
    /// No automatic reconnection
    Manual,
}

impl ConnectionManager {
    /// Handle connection loss
    pub async fn handle_disconnection(
        &self,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    ) -> Result<()> {
        let connection = self.pool.get(&connection_id).await?;
        
        match self.reconnection {
            ReconnectionStrategy::Stateful { resume_window, ref state_store } => {
                // Save connection state
                let state = connection.capture_state().await?;
                let resume_token = ResumeToken::new();
                
                state_store.save(resume_token.clone(), state, resume_window).await?;
                
                // Mark as suspended
                connection.suspend(resume_token).await?;
                
                // Notify participant
                self.notify_suspension(connection.participant_id, resume_token).await?;
            }
            
            ReconnectionStrategy::ExponentialBackoff { .. } => {
                // Schedule reconnection attempts
                self.schedule_reconnection(connection_id).await?;
            }
            
            ReconnectionStrategy::Manual => {
                // Clean up immediately
                self.pool.remove(&connection_id).await?;
            }
        }
        
        Ok(())
    }
    
    /// Resume suspended connection
    pub async fn resume_connection(
        &self,
        resume_token: ResumeToken,
        new_ws: WebSocket,
    ) -> Result<ConnectionId> {
        // Retrieve saved state
        let state = self.sessions.retrieve_state(resume_token).await?;
        
        // Create new connection with old state
        let connection_id = ConnectionId::new();
        let connection = self.create_connection_with_state(new_ws, state).await?;
        
        // Register new connection
        self.pool.add(connection_id.clone(), connection).await?;
        
        // Send missed messages
        self.send_missed_messages(connection_id.clone(), state.last_message_id).await?;
        
        Ok(connection_id)
    }
}
```

### Scaling Patterns

Horizontal scaling strategies:

```rust
pub struct ScalableHub {
    /// Load balancer for incoming connections
    load_balancer: Arc<LoadBalancer>,
    
    /// Hub instances
    instances: Vec<Arc<HubInstance>>,
    
    /// Shared state backend
    state_backend: Arc<StateBackend>,
    
    /// Inter-hub communication
    hub_mesh: Arc<HubMesh>,
}

pub struct LoadBalancer {
    /// Load balancing algorithm
    algorithm: LoadBalancingAlgorithm,
    
    /// Health checks
    health_checker: HealthChecker,
}

pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    
    /// Least connections
    LeastConnections,
    
    /// Weighted by capacity
    WeightedCapacity,
    
    /// Geolocation-based
    Geographic,
    
    /// Consistent hashing for sticky sessions
    ConsistentHash,
}

pub struct HubMesh {
    /// Inter-hub message bus
    message_bus: Arc<MessageBus>,
    
    /// Hub discovery
    discovery: Arc<HubDiscovery>,
    
    /// Gossip protocol for state sync
    gossip: Arc<GossipProtocol>,
}

impl ScalableHub {
    /// Scale up by adding hub instance
    pub async fn scale_up(&self) -> Result<()> {
        // Create new hub instance
        let instance = HubInstance::new(self.state_backend.clone()).await?;
        
        // Register with load balancer
        self.load_balancer.register_instance(instance.clone()).await?;
        
        // Join hub mesh
        self.hub_mesh.add_node(instance.clone()).await?;
        
        // Start accepting connections
        instance.start().await?;
        
        self.instances.push(instance);
        
        Ok(())
    }
    
    /// Redistribute connections during scale down
    pub async fn scale_down(&self, instance_id: InstanceId) -> Result<()> {
        let instance = self.find_instance(instance_id)?;
        
        // Stop accepting new connections
        self.load_balancer.deregister_instance(instance_id).await?;
        
        // Migrate existing connections
        let connections = instance.get_connections().await?;
        
        for connection in connections {
            let target_instance = self.load_balancer.select_instance().await?;
            self.migrate_connection(connection, instance, target_instance).await?;
        }
        
        // Leave hub mesh
        self.hub_mesh.remove_node(instance_id).await?;
        
        // Shutdown instance
        instance.shutdown().await?;
        
        Ok(())
    }
}
```

### Message Persistence and Recovery

Ensuring message delivery and history:

```rust
pub struct MessagePersistence {
    /// Message store
    store: Arc<dyn MessageStore>,
    
    /// Delivery tracking
    delivery_tracker: Arc<DeliveryTracker>,
    
    /// Retention policy
    retention: RetentionPolicy,
}

pub trait MessageStore: Send + Sync {
    async fn store(&self, message: Message) -> Result<MessageId>;
    async fn retrieve(&self, id: MessageId) -> Result<Option<Message>>;
    async fn query(&self, filter: MessageFilter) -> Result<Vec<Message>>;
    async fn delete(&self, id: MessageId) -> Result<()>;
}

pub struct DeliveryTracker {
    /// Pending deliveries
    pending: Arc<RwLock<HashMap<MessageId, PendingDelivery>>>,
    
    /// Acknowledgment timeout
    ack_timeout: Duration,
    
    /// Retry policy
    retry_policy: RetryPolicy,
}

pub struct PendingDelivery {
    message_id: MessageId,
    recipients: HashSet<ParticipantId>,
    attempts: u32,
    next_retry: Instant,
}

impl MessagePersistence {
    /// Store and track message delivery
    pub async fn persist_and_deliver(
        &self,
        message: Message,
        recipients: Vec<ParticipantId>,
    ) -> Result<DeliveryStatus> {
        // Store message
        let message_id = self.store.store(message.clone()).await?;
        
        // Track delivery
        let pending = PendingDelivery {
            message_id: message_id.clone(),
            recipients: recipients.into_iter().collect(),
            attempts: 0,
            next_retry: Instant::now(),
        };
        
        self.delivery_tracker.track(pending).await?;
        
        // Attempt delivery
        let results = self.deliver_to_recipients(message, &pending.recipients).await?;
        
        // Update tracking based on results
        for (recipient, result) in results {
            if result.is_success() {
                self.delivery_tracker.mark_delivered(message_id.clone(), recipient).await?;
            }
        }
        
        Ok(self.delivery_tracker.get_status(message_id).await?)
    }
}
```

### Security and Authentication

Securing WebSocket connections:

```rust
pub struct WebSocketSecurity {
    /// Authentication handler
    authenticator: Arc<dyn Authenticator>,
    
    /// Authorization handler
    authorizer: Arc<dyn Authorizer>,
    
    /// Encryption layer
    encryption: Option<EncryptionLayer>,
    
    /// Rate limiting
    rate_limiter: Arc<RateLimiter>,
}

pub struct EncryptionLayer {
    /// TLS configuration
    tls_config: TlsConfig,
    
    /// Message encryption
    message_encryption: MessageEncryption,
}

pub enum MessageEncryption {
    /// No additional encryption (rely on TLS)
    None,
    
    /// End-to-end encryption
    E2E {
        key_exchange: KeyExchangeProtocol,
        cipher: CipherSuite,
    },
    
    /// Selective field encryption
    Selective {
        fields: Vec<String>,
        cipher: CipherSuite,
    },
}

impl WebSocketSecurity {
    /// Authenticate new connection
    pub async fn authenticate_connection(
        &self,
        connection: &WebSocketConnection,
        credentials: Credentials,
    ) -> Result<AuthenticationResult> {
        // Check rate limits
        self.rate_limiter.check(connection.remote_addr()).await?;
        
        // Authenticate
        let identity = self.authenticator.authenticate(credentials).await?;
        
        // Create session
        let session = Session {
            identity: identity.clone(),
            permissions: self.authorizer.get_permissions(&identity).await?,
            expires_at: Utc::now() + Duration::hours(24),
        };
        
        Ok(AuthenticationResult {
            identity,
            session,
            token: self.generate_session_token(),
        })
    }
    
    /// Authorize message operation
    pub async fn authorize_operation(
        &self,
        participant: &ParticipantId,
        operation: Operation,
    ) -> Result<bool> {
        self.authorizer.check_permission(participant, operation).await
    }
}
```

## Performance Optimization

### Message Batching and Compression

Optimizing bandwidth usage:

```rust
pub struct MessageOptimizer {
    /// Batching configuration
    batching: BatchingConfig,
    
    /// Compression settings
    compression: CompressionConfig,
    
    /// Delta encoding for updates
    delta_encoding: bool,
}

pub struct BatchingConfig {
    /// Maximum batch size
    max_size: usize,
    
    /// Maximum wait time
    max_wait: Duration,
    
    /// Batching strategy
    strategy: BatchingStrategy,
}

pub enum CompressionConfig {
    /// No compression
    None,
    
    /// Standard compression
    Standard {
        algorithm: CompressionAlgorithm,
        level: u32,
    },
    
    /// Adaptive compression
    Adaptive {
        threshold: usize,
        algorithms: Vec<CompressionAlgorithm>,
    },
}

impl MessageOptimizer {
    pub async fn optimize_messages(
        &self,
        messages: Vec<Message>,
    ) -> Result<OptimizedPayload> {
        let mut payload = OptimizedPayload::new();
        
        // Batch messages
        if messages.len() > 1 && self.batching.should_batch(&messages) {
            payload = self.create_batch(messages)?;
        } else {
            payload.messages = messages;
        }
        
        // Apply compression
        if let Some(compressed) = self.compress(&payload)? {
            payload = compressed;
        }
        
        // Apply delta encoding
        if self.delta_encoding {
            payload = self.apply_delta_encoding(payload)?;
        }
        
        Ok(payload)
    }
}
```

### Connection Pooling and Reuse

Efficient connection management:

```rust
pub struct ConnectionPoolManager {
    /// Pool configuration
    config: PoolConfig,
    
    /// Active pools by endpoint
    pools: Arc<RwLock<HashMap<Endpoint, ConnectionPool>>>,
    
    /// Health monitoring
    health_monitor: Arc<HealthMonitor>,
}

pub struct PoolConfig {
    /// Minimum connections to maintain
    min_idle: usize,
    
    /// Maximum connections allowed
    max_size: usize,
    
    /// Connection timeout
    connection_timeout: Duration,
    
    /// Idle timeout before closing
    idle_timeout: Duration,
    
    /// Validation interval
    validation_interval: Duration,
}

impl ConnectionPoolManager {
    pub async fn get_connection(&self, endpoint: &Endpoint) -> Result<PooledConnection> {
        let pool = self.get_or_create_pool(endpoint).await?;
        
        // Try to get existing connection
        if let Some(conn) = pool.try_get().await {
            if self.validate_connection(&conn).await {
                return Ok(conn);
            }
        }
        
        // Create new connection if needed
        if pool.size() < self.config.max_size {
            let conn = self.create_connection(endpoint).await?;
            pool.add(conn.clone()).await?;
            return Ok(conn);
        }
        
        // Wait for available connection
        pool.wait_for_available(self.config.connection_timeout).await
    }
}
```

## Monitoring and Observability

### Metrics Collection

Key metrics for hub health:

```rust
pub struct HubMetrics {
    /// Connection metrics
    connections: ConnectionMetrics,
    
    /// Message metrics
    messages: MessageMetrics,
    
    /// Performance metrics
    performance: PerformanceMetrics,
    
    /// Error metrics
    errors: ErrorMetrics,
}

pub struct ConnectionMetrics {
    /// Current active connections
    pub active: Gauge,
    
    /// Total connections created
    pub total: Counter,
    
    /// Connection duration histogram
    pub duration: Histogram,
    
    /// Disconnection reasons
    pub disconnection_reasons: HashMap<DisconnectReason, Counter>,
}

pub struct MessageMetrics {
    /// Messages sent per second
    pub send_rate: Gauge,
    
    /// Messages received per second
    pub receive_rate: Gauge,
    
    /// Message size distribution
    pub size_distribution: Histogram,
    
    /// Broadcast latency
    pub broadcast_latency: Histogram,
}

impl HubMetrics {
    pub fn record_connection(&self, event: ConnectionEvent) {
        match event {
            ConnectionEvent::Connected => {
                self.connections.active.inc();
                self.connections.total.inc();
            }
            ConnectionEvent::Disconnected(reason) => {
                self.connections.active.dec();
                self.connections.disconnection_reasons
                    .entry(reason)
                    .or_insert_with(Counter::new)
                    .inc();
            }
        }
    }
}
```

## Best Practices

### Connection Management

1. **Heartbeat/Ping-Pong**: Implement regular heartbeats to detect dead connections
2. **Graceful Shutdown**: Notify clients before closing connections
3. **Resume Tokens**: Allow clients to resume after temporary disconnections
4. **Connection Limits**: Enforce per-client and global connection limits

### Message Handling

1. **Message Validation**: Validate all incoming messages
2. **Size Limits**: Enforce maximum message sizes
3. **Rate Limiting**: Prevent message flooding
4. **Delivery Guarantees**: Implement at-least-once or exactly-once delivery

### Scalability

1. **Horizontal Scaling**: Design for multiple hub instances
2. **State Management**: Use external state stores for shared state
3. **Load Distribution**: Balance connections across instances
4. **Graceful Degradation**: Handle partial failures

### Security

1. **Authentication**: Require authentication before accepting messages
2. **Authorization**: Check permissions for each operation
3. **Encryption**: Use TLS and consider E2E encryption
4. **Input Sanitization**: Sanitize all user input

## Relationships

- **Parent Nodes:** [elements/distributed_conversation_coordination.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/protocol_based_exposure.md] - implements - WebSocket protocol
  - [elements/interruptible_agent_loops.md] - uses - Real-time messaging
  - [elements/async_human_in_loop.md] - enables - Human participation

## Conclusion

WebSocket hub architecture provides the foundation for real-time, multi-participant agent conversations. By implementing proper connection management, message routing, scaling patterns, and security measures, WebSocket hubs can support dynamic conversations with arbitrary numbers of participants while maintaining performance and reliability. The patterns described here enable building systems that scale from simple single-hub deployments to complex federated networks supporting millions of concurrent connections.