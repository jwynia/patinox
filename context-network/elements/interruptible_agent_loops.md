# Interruptible Agent Loops

## Overview

Traditional agent frameworks often treat agent execution as an atomic operation - once started, the agent runs to completion. However, real-world scenarios require agents that can be interrupted, receive new instructions mid-execution, and gracefully handle priority changes. This document explores patterns for building agent loops that can be interrupted while maintaining consistency and state integrity.

## Core Concepts

### The Agent Loop Pattern

Most agents operate in a loop with their tools:

```rust
pub struct AgentLoop {
    // Core agent state
    agent: Box<dyn Agent>,
    
    // Available tools
    tools: ToolRegistry,
    
    // Message queue for interruptions
    message_queue: Arc<Mutex<MessageQueue>>,
    
    // Interruption handler
    interrupt_handler: InterruptHandler,
    
    // Loop control
    control: LoopControl,
}

pub struct LoopControl {
    pub running: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    pub interrupt_pending: Arc<AtomicBool>,
    pub priority_changed: Arc<AtomicBool>,
}

impl AgentLoop {
    pub async fn run(&mut self) -> Result<LoopResult> {
        while self.control.running.load(Ordering::Relaxed) {
            // Check for interruptions
            if self.control.interrupt_pending.load(Ordering::Relaxed) {
                self.handle_interrupt().await?;
            }
            
            // Check if paused
            while self.control.paused.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
                self.check_messages().await?;
            }
            
            // Execute next agent step
            let action = self.agent.next_action().await?;
            
            // Check for interruption before tool execution
            if self.should_interrupt_before_action(&action) {
                self.handle_interrupt().await?;
            }
            
            // Execute action with tools
            let result = self.execute_with_tools(action).await?;
            
            // Update agent state
            self.agent.observe(result).await?;
            
            // Check completion
            if self.agent.is_complete() {
                return Ok(LoopResult::Completed(self.agent.get_result()));
            }
        }
        
        Ok(LoopResult::Interrupted)
    }
}
```

### Interruption Points

Strategic locations where interruptions can be safely handled:

```rust
pub enum InterruptionPoint {
    // Before starting a new reasoning cycle
    BeforeReasoning,
    
    // After reasoning but before action
    BeforeAction,
    
    // After action execution
    AfterAction,
    
    // During long-running operations
    DuringOperation {
        checkpoint_interval: Duration,
    },
    
    // At explicit yield points
    ExplicitYield,
}

pub struct InterruptibleOperation {
    pub operation: Box<dyn Operation>,
    pub checkpoints: Vec<CheckpointLocation>,
    pub can_resume: bool,
}

impl InterruptibleOperation {
    pub async fn execute_with_interruption(&self) -> Result<OperationResult> {
        let mut progress = Progress::new();
        
        for checkpoint in &self.checkpoints {
            // Execute to next checkpoint
            let partial = self.operation.execute_partial(&progress, checkpoint).await?;
            progress.update(partial);
            
            // Check for interruption
            if self.check_interrupt_signal() {
                if self.can_resume {
                    return Ok(OperationResult::Suspended(progress));
                } else {
                    return Ok(OperationResult::MustComplete(
                        self.operation.complete_from(progress).await?
                    ));
                }
            }
        }
        
        Ok(OperationResult::Completed(progress.finalize()))
    }
}
```

### Message-Based Interruption

Agents receive messages that can alter their execution:

```rust
pub enum InterruptMessage {
    // Pause execution
    Pause {
        reason: String,
        save_state: bool,
    },
    
    // Resume execution
    Resume {
        context: Option<ResumeContext>,
    },
    
    // Change priority or focus
    ChangePriority {
        new_task: Option<Task>,
        priority: Priority,
        abandon_current: bool,
    },
    
    // Inject new information
    UpdateContext {
        information: Information,
        integration: IntegrationStrategy,
    },
    
    // Request status
    StatusRequest {
        response_channel: Sender<Status>,
    },
    
    // Emergency stop
    EmergencyStop {
        cleanup: CleanupStrategy,
    },
}

pub struct MessageHandler {
    pub async fn handle_message(&mut self, msg: InterruptMessage) -> Result<()> {
        match msg {
            InterruptMessage::Pause { save_state, .. } => {
                if save_state {
                    self.checkpoint_state().await?;
                }
                self.control.paused.store(true, Ordering::Relaxed);
            }
            
            InterruptMessage::ChangePriority { new_task, abandon_current, .. } => {
                if abandon_current {
                    self.abandon_current_task().await?;
                }
                if let Some(task) = new_task {
                    self.queue_high_priority(task).await?;
                }
            }
            
            InterruptMessage::UpdateContext { information, integration } => {
                match integration {
                    IntegrationStrategy::Immediate => {
                        self.agent.inject_context(information).await?;
                    }
                    IntegrationStrategy::AtNextCycle => {
                        self.pending_context.push(information);
                    }
                }
            }
            
            // ... other message types
        }
        Ok(())
    }
}
```

## Tool Integration Patterns

### Tool Execution with Interruption Support

```rust
pub struct InterruptibleToolExecutor {
    tools: HashMap<String, Box<dyn Tool>>,
    interrupt_checker: Arc<dyn InterruptChecker>,
}

impl InterruptibleToolExecutor {
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: ToolParams,
    ) -> Result<ToolResult> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| Error::ToolNotFound(tool_name.to_string()))?;
        
        // Check if tool supports interruption
        if let Some(interruptible) = tool.as_interruptible() {
            self.execute_interruptible(interruptible, params).await
        } else {
            // Non-interruptible tools run to completion
            self.execute_atomic(tool.as_ref(), params).await
        }
    }
    
    async fn execute_interruptible(
        &self,
        tool: &dyn InterruptibleTool,
        params: ToolParams,
    ) -> Result<ToolResult> {
        let mut state = tool.initialize(params).await?;
        
        loop {
            // Execute a chunk of work
            let progress = tool.execute_chunk(&mut state).await?;
            
            // Check for interruption
            if self.interrupt_checker.should_interrupt().await {
                // Save tool state for resumption
                let suspended = tool.suspend(state).await?;
                return Ok(ToolResult::Suspended(suspended));
            }
            
            if progress.is_complete() {
                return Ok(ToolResult::Completed(progress.result));
            }
            
            state = progress.next_state;
        }
    }
}
```

### Tool State Management

```rust
pub trait InterruptibleTool: Tool {
    type State: Serialize + DeserializeOwned;
    
    async fn initialize(&self, params: ToolParams) -> Result<Self::State>;
    
    async fn execute_chunk(&self, state: &mut Self::State) -> Result<Progress>;
    
    async fn suspend(&self, state: Self::State) -> Result<SuspendedState>;
    
    async fn resume(&self, suspended: SuspendedState) -> Result<Self::State>;
}

pub struct Progress {
    pub result: Option<serde_json::Value>,
    pub percentage: f64,
    pub next_state: State,
    pub can_interrupt_after: bool,
}

// Example: Interruptible web scraping tool
pub struct WebScrapingTool {
    client: HttpClient,
    chunk_size: usize,
}

impl InterruptibleTool for WebScrapingTool {
    type State = ScrapingState;
    
    async fn execute_chunk(&self, state: &mut Self::State) -> Result<Progress> {
        let batch = state.get_next_batch(self.chunk_size);
        let results = Vec::new();
        
        for url in batch {
            // Scrape individual page
            let content = self.client.get(&url).await?;
            results.push(self.parse_content(content)?);
            state.mark_complete(url);
        }
        
        state.add_results(results);
        
        Ok(Progress {
            result: if state.is_complete() { 
                Some(state.aggregate_results()) 
            } else { 
                None 
            },
            percentage: state.completion_percentage(),
            next_state: state.clone(),
            can_interrupt_after: true,
        })
    }
}
```

## Priority Management

### Dynamic Priority Adjustment

```rust
pub struct PriorityManager {
    current_task: Option<Task>,
    task_queue: PriorityQueue<Task>,
    preemption_policy: PreemptionPolicy,
}

pub enum PreemptionPolicy {
    // Never preempt current task
    NoPreemption,
    
    // Preempt if higher priority
    PriorityBased {
        threshold: Priority,
    },
    
    // Preempt based on estimated time
    TimeAware {
        max_delay: Duration,
    },
    
    // Complex policy
    Custom(Box<dyn PreemptionLogic>),
}

impl PriorityManager {
    pub async fn handle_new_task(&mut self, task: Task) -> TaskHandling {
        let priority = task.priority();
        
        match self.preemption_policy {
            PreemptionPolicy::NoPreemption => {
                self.task_queue.push(task);
                TaskHandling::Queued
            }
            
            PreemptionPolicy::PriorityBased { threshold } => {
                if priority > threshold && self.current_task.is_some() {
                    let current = self.current_task.take().unwrap();
                    self.task_queue.push(current);
                    self.current_task = Some(task);
                    TaskHandling::Preempted
                } else {
                    self.task_queue.push(task);
                    TaskHandling::Queued
                }
            }
            
            // ... other policies
        }
    }
}
```

### Task Switching

```rust
pub struct TaskSwitcher {
    save_context: bool,
    switching_strategy: SwitchingStrategy,
}

pub enum SwitchingStrategy {
    // Complete current operation first
    GracefulSwitch {
        max_wait: Duration,
    },
    
    // Switch immediately
    ImmediateSwitch,
    
    // Switch at next checkpoint
    CheckpointSwitch,
}

impl TaskSwitcher {
    pub async fn switch_task(
        &self,
        from: &mut Task,
        to: Task,
        agent: &mut Agent,
    ) -> Result<()> {
        match self.switching_strategy {
            SwitchingStrategy::GracefulSwitch { max_wait } => {
                let deadline = Instant::now() + max_wait;
                
                // Try to reach a good stopping point
                while Instant::now() < deadline {
                    if agent.at_stopping_point() {
                        break;
                    }
                    agent.step().await?;
                }
            }
            
            SwitchingStrategy::ImmediateSwitch => {
                // Stop immediately
            }
            
            SwitchingStrategy::CheckpointSwitch => {
                // Continue to next checkpoint
                agent.continue_to_checkpoint().await?;
            }
        }
        
        if self.save_context {
            from.save_context(agent.get_context()).await?;
        }
        
        // Load new task context
        if let Some(context) = to.get_saved_context() {
            agent.load_context(context).await?;
        }
        
        agent.set_current_task(to);
        Ok(())
    }
}
```

## State Preservation

### Context Snapshots

```rust
pub struct ContextSnapshot {
    pub timestamp: DateTime<Utc>,
    pub task_id: TaskId,
    pub agent_state: AgentState,
    pub tool_states: HashMap<String, ToolState>,
    pub partial_results: Vec<PartialResult>,
    pub message_queue: Vec<InterruptMessage>,
}

impl ContextSnapshot {
    pub fn capture(agent_loop: &AgentLoop) -> Self {
        Self {
            timestamp: Utc::now(),
            task_id: agent_loop.current_task_id(),
            agent_state: agent_loop.agent.get_state(),
            tool_states: agent_loop.capture_tool_states(),
            partial_results: agent_loop.get_partial_results(),
            message_queue: agent_loop.get_pending_messages(),
        }
    }
    
    pub async fn restore(&self, agent_loop: &mut AgentLoop) -> Result<()> {
        agent_loop.agent.set_state(self.agent_state.clone())?;
        
        for (tool_name, state) in &self.tool_states {
            agent_loop.restore_tool_state(tool_name, state).await?;
        }
        
        agent_loop.set_partial_results(self.partial_results.clone());
        agent_loop.queue_messages(self.message_queue.clone());
        
        Ok(())
    }
}
```

### Continuation Tokens

```rust
pub struct ContinuationToken {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub encrypted_state: Vec<u8>,
    pub checksum: Hash,
}

impl ContinuationToken {
    pub fn create(snapshot: &ContextSnapshot, encryption_key: &Key) -> Result<Self> {
        let serialized = bincode::serialize(snapshot)?;
        let encrypted = encrypt(&serialized, encryption_key)?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
            encrypted_state: encrypted,
            checksum: calculate_hash(&serialized),
        })
    }
    
    pub fn restore(&self, encryption_key: &Key) -> Result<ContextSnapshot> {
        if Utc::now() > self.expires_at {
            return Err(Error::TokenExpired);
        }
        
        let decrypted = decrypt(&self.encrypted_state, encryption_key)?;
        let snapshot: ContextSnapshot = bincode::deserialize(&decrypted)?;
        
        // Verify integrity
        if calculate_hash(&decrypted) != self.checksum {
            return Err(Error::IntegrityCheckFailed);
        }
        
        Ok(snapshot)
    }
}
```

## Coordination Patterns

### Multi-Agent Interruption

```rust
pub struct MultiAgentCoordinator {
    agents: HashMap<AgentId, AgentLoop>,
    interrupt_propagation: PropagationStrategy,
}

pub enum PropagationStrategy {
    // Interrupt all agents
    Broadcast,
    
    // Interrupt dependent agents
    Cascade {
        dependency_graph: DependencyGraph,
    },
    
    // Selective interruption
    Selective {
        selector: Box<dyn AgentSelector>,
    },
}

impl MultiAgentCoordinator {
    pub async fn propagate_interrupt(
        &mut self,
        source: AgentId,
        interrupt: InterruptMessage,
    ) -> Result<()> {
        match self.interrupt_propagation {
            PropagationStrategy::Broadcast => {
                for (id, agent) in &mut self.agents {
                    if *id != source {
                        agent.send_interrupt(interrupt.clone()).await?;
                    }
                }
            }
            
            PropagationStrategy::Cascade { ref dependency_graph } => {
                let affected = dependency_graph.get_dependents(&source);
                for agent_id in affected {
                    if let Some(agent) = self.agents.get_mut(&agent_id) {
                        agent.send_interrupt(interrupt.clone()).await?;
                    }
                }
            }
            
            // ... other strategies
        }
        
        Ok(())
    }
}
```

## Graceful Shutdown

### Cleanup Strategies

```rust
pub struct ShutdownManager {
    timeout: Duration,
    cleanup_strategy: CleanupStrategy,
}

pub enum CleanupStrategy {
    // Save everything
    Complete,
    
    // Save critical state only
    Critical,
    
    // Best effort within timeout
    BestEffort,
    
    // Immediate termination
    Immediate,
}

impl ShutdownManager {
    pub async fn shutdown(&self, agent_loop: &mut AgentLoop) -> Result<ShutdownResult> {
        let deadline = Instant::now() + self.timeout;
        
        match self.cleanup_strategy {
            CleanupStrategy::Complete => {
                // Signal shutdown
                agent_loop.initiate_shutdown().await?;
                
                // Wait for current operation
                agent_loop.complete_current_operation().await?;
                
                // Save all state
                let snapshot = ContextSnapshot::capture(agent_loop);
                self.persist_snapshot(snapshot).await?;
                
                // Cleanup resources
                agent_loop.cleanup_resources().await?;
            }
            
            CleanupStrategy::BestEffort => {
                let remaining = deadline - Instant::now();
                
                // Try to save state within timeout
                match timeout(remaining, agent_loop.save_state()).await {
                    Ok(Ok(_)) => {},
                    _ => {
                        // Log failure but continue
                    }
                }
            }
            
            // ... other strategies
        }
        
        Ok(ShutdownResult::Success)
    }
}
```

## Performance Considerations

### Interrupt Checking Overhead

```rust
pub struct InterruptOptimizer {
    check_frequency: CheckFrequency,
    batch_size: usize,
}

pub enum CheckFrequency {
    // Check every operation
    Always,
    
    // Check periodically
    Periodic(Duration),
    
    // Check after N operations
    OperationCount(usize),
    
    // Adaptive based on load
    Adaptive,
}

impl InterruptOptimizer {
    pub fn should_check(&mut self) -> bool {
        match self.check_frequency {
            CheckFrequency::Always => true,
            CheckFrequency::Periodic(duration) => {
                self.last_check.elapsed() > duration
            }
            CheckFrequency::OperationCount(n) => {
                self.operation_count % n == 0
            }
            CheckFrequency::Adaptive => {
                self.calculate_adaptive_check()
            }
        }
    }
}
```

## Integration with Other Patterns

### With Resumable Workflows
- Interruption points align with checkpoint locations
- State snapshots compatible with workflow checkpoints
- Continuation tokens enable cross-session resumption

### With Dual-Context Evaluation
- Evaluation context can trigger interruptions
- Generation can be interrupted for re-evaluation
- Priority changes based on evaluation results

### With Human-in-the-Loop
- Human messages trigger interruptions
- Agents pause for human approval
- Priority changes from human input

## Best Practices

### Interruption Design
1. Define clear interruption points
2. Minimize state that needs preservation
3. Make operations idempotent where possible
4. Test interruption at every defined point

### Message Handling
1. Use bounded queues to prevent overflow
2. Prioritize messages by importance
3. Batch similar messages when possible
4. Log all interruption events

### State Management
1. Keep state serializable
2. Version state structures for compatibility
3. Implement state validation on restore
4. Regular cleanup of old snapshots

### Performance
1. Balance interruption checking with performance
2. Use async operations for state preservation
3. Implement timeout for cleanup operations
4. Monitor interruption handling metrics

## Conclusion

Interruptible agent loops enable responsive, flexible agent systems that can adapt to changing priorities and requirements in real-time. By carefully designing interruption points, message handling, and state preservation mechanisms, we can build agents that maintain consistency while remaining responsive to external events and priority changes.