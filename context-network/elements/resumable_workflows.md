# Resumable Workflows Architecture

## Overview

Resumable workflows are essential for building robust agent systems that can survive failures, interruptions, and long-running operations. This architecture enables agents to checkpoint their state, pause execution, and resume from exactly where they left off - whether due to system failures, resource constraints, or human intervention requirements.

## Core Concepts

### Workflow State Persistence

Every workflow maintains a complete snapshot of its execution state that can be serialized and restored:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowState {
    // Unique identifier for this workflow instance
    pub workflow_id: Uuid,
    
    // Current execution phase
    pub phase: WorkflowPhase,
    
    // Completed steps with their results
    pub completed_steps: Vec<CompletedStep>,
    
    // Pending steps to be executed
    pub pending_steps: VecDeque<Step>,
    
    // Current step being executed (if any)
    pub current_step: Option<ExecutingStep>,
    
    // Accumulated context from all steps
    pub context: WorkflowContext,
    
    // Checkpoint metadata
    pub last_checkpoint: CheckpointMetadata,
    
    // Paradigm-specific state
    pub paradigm_state: ParadigmState,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum WorkflowPhase {
    Initializing,
    Planning,
    Executing,
    Suspended,
    WaitingForApproval,
    Evaluating,
    Completed,
    Failed(String),
}
```

### Checkpoint Mechanisms

Checkpoints capture the complete workflow state at critical points:

```rust
pub trait CheckpointManager {
    async fn create_checkpoint(&self, state: &WorkflowState) -> Result<CheckpointId>;
    async fn restore_checkpoint(&self, id: CheckpointId) -> Result<WorkflowState>;
    async fn list_checkpoints(&self, workflow_id: Uuid) -> Result<Vec<CheckpointInfo>>;
    async fn prune_old_checkpoints(&self, keep_last: usize) -> Result<()>;
}

pub struct CheckpointStrategy {
    // When to create checkpoints
    pub triggers: Vec<CheckpointTrigger>,
    
    // Storage backend
    pub storage: Box<dyn CheckpointStorage>,
    
    // Compression and encryption
    pub compression: CompressionType,
    pub encryption: Option<EncryptionConfig>,
}

pub enum CheckpointTrigger {
    // Time-based
    Interval(Duration),
    
    // Progress-based
    AfterSteps(usize),
    
    // Event-based
    BeforeRiskyOperation,
    AfterExpensiveOperation,
    OnStateTransition,
    
    // Resource-based
    MemoryThreshold(usize),
    TokenThreshold(usize),
}
```

### Event Sourcing

All workflow events are recorded in an append-only log for complete auditability:

```rust
#[derive(Serialize, Deserialize)]
pub enum WorkflowEvent {
    Started {
        workflow_id: Uuid,
        initial_input: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    StepCompleted {
        step_id: Uuid,
        result: StepResult,
        duration: Duration,
    },
    Checkpointed {
        checkpoint_id: CheckpointId,
        size_bytes: usize,
    },
    Suspended {
        reason: SuspensionReason,
        can_resume: bool,
    },
    Resumed {
        checkpoint_id: CheckpointId,
        resume_context: Option<serde_json::Value>,
    },
    Failed {
        error: String,
        recoverable: bool,
    },
}

pub struct EventStore {
    events: Vec<WorkflowEvent>,
    
    pub async fn append(&mut self, event: WorkflowEvent) -> Result<()> {
        self.events.push(event);
        self.persist().await
    }
    
    pub async fn replay_from(&self, checkpoint: Option<CheckpointId>) -> Result<WorkflowState> {
        // Reconstruct state by replaying events
    }
}
```

## Suspension and Resumption Patterns

### Graceful Suspension

Workflows can be suspended at safe points:

```rust
pub struct SuspendableWorkflow {
    suspension_points: Vec<SuspensionPoint>,
    suspension_requested: Arc<AtomicBool>,
}

impl SuspendableWorkflow {
    pub async fn execute_with_suspension(&mut self) -> Result<WorkflowResult> {
        loop {
            // Check for suspension request
            if self.suspension_requested.load(Ordering::Relaxed) {
                return self.suspend().await;
            }
            
            // Execute next step
            let step = self.get_next_step()?;
            
            // Check if this is a suspension point
            if self.is_suspension_point(&step) {
                self.create_checkpoint().await?;
                
                if self.should_suspend(&step) {
                    return self.suspend().await;
                }
            }
            
            let result = self.execute_step(step).await?;
            self.record_result(result).await?;
            
            if self.is_complete() {
                return Ok(WorkflowResult::Completed(self.get_output()));
            }
        }
    }
}
```

### Resume Strategies

Different strategies for resuming workflows based on the suspension reason:

```rust
pub enum ResumeStrategy {
    // Continue from exact suspension point
    ExactContinuation,
    
    // Re-evaluate current step
    RetryCurrentStep,
    
    // Rollback to previous checkpoint
    RollbackToCheckpoint(CheckpointId),
    
    // Start fresh with accumulated knowledge
    RestartWithContext,
    
    // Skip failed step and continue
    SkipAndContinue,
}

pub struct WorkflowResumer {
    pub async fn resume(
        &self,
        workflow_id: Uuid,
        strategy: ResumeStrategy,
        context: Option<ResumeContext>,
    ) -> Result<WorkflowHandle> {
        let state = match strategy {
            ResumeStrategy::ExactContinuation => {
                self.restore_latest_state(workflow_id).await?
            }
            ResumeStrategy::RetryCurrentStep => {
                let mut state = self.restore_latest_state(workflow_id).await?;
                state.retry_current_step();
                state
            }
            ResumeStrategy::RollbackToCheckpoint(id) => {
                self.restore_checkpoint(id).await?
            }
            ResumeStrategy::RestartWithContext => {
                self.create_fresh_with_context(workflow_id, context).await?
            }
            ResumeStrategy::SkipAndContinue => {
                let mut state = self.restore_latest_state(workflow_id).await?;
                state.skip_current_step();
                state
            }
        };
        
        Ok(WorkflowHandle::resumed(state))
    }
}
```

## Saga Pattern Implementation

Long-running transactions with compensating actions:

```rust
pub struct Saga {
    pub steps: Vec<SagaStep>,
    pub compensation_strategy: CompensationStrategy,
}

pub struct SagaStep {
    pub id: Uuid,
    pub action: Box<dyn Action>,
    pub compensation: Box<dyn CompensatingAction>,
    pub retry_policy: RetryPolicy,
}

pub enum CompensationStrategy {
    // Compensate all completed steps in reverse order
    FullRollback,
    
    // Compensate only steps after failure point
    PartialRollback(usize),
    
    // Custom compensation logic
    Custom(Box<dyn CompensationLogic>),
}

impl Saga {
    pub async fn execute(&self) -> Result<SagaResult> {
        let mut completed_steps = Vec::new();
        
        for step in &self.steps {
            match self.execute_step_with_retry(step).await {
                Ok(result) => {
                    completed_steps.push((step, result));
                }
                Err(e) => {
                    // Step failed, initiate compensation
                    return self.compensate(completed_steps, e).await;
                }
            }
        }
        
        Ok(SagaResult::Success(completed_steps))
    }
    
    async fn compensate(
        &self,
        completed_steps: Vec<(&SagaStep, StepResult)>,
        error: Error,
    ) -> Result<SagaResult> {
        match self.compensation_strategy {
            CompensationStrategy::FullRollback => {
                for (step, _) in completed_steps.iter().rev() {
                    step.compensation.execute().await?;
                }
            }
            // ... other strategies
        }
        
        Ok(SagaResult::Compensated { error, compensated_steps })
    }
}
```

## Integration with Human-in-the-Loop

Workflows can pause for human approval and resume when decisions are made:

```rust
pub struct ApprovalPoint {
    pub id: Uuid,
    pub description: String,
    pub required_approver: ApproverRole,
    pub timeout: Option<Duration>,
    pub fallback_action: FallbackAction,
}

pub enum FallbackAction {
    Fail,
    Skip,
    AutoApprove,
    Escalate(ApproverRole),
}

impl WorkflowState {
    pub async fn wait_for_approval(&mut self, point: ApprovalPoint) -> Result<ApprovalDecision> {
        // Create checkpoint before waiting
        self.create_checkpoint().await?;
        
        // Update state to waiting
        self.phase = WorkflowPhase::WaitingForApproval;
        
        // Create approval request
        let request = ApprovalRequest {
            workflow_id: self.workflow_id,
            approval_point: point.clone(),
            context: self.get_approval_context(),
            created_at: Utc::now(),
        };
        
        // Send to approval queue
        self.send_approval_request(request).await?;
        
        // Suspend workflow
        self.suspend_for_approval().await
    }
    
    pub async fn resume_with_approval(&mut self, decision: ApprovalDecision) -> Result<()> {
        match decision {
            ApprovalDecision::Approved(context) => {
                self.context.merge(context);
                self.phase = WorkflowPhase::Executing;
            }
            ApprovalDecision::Rejected(reason) => {
                self.phase = WorkflowPhase::Failed(reason);
            }
            ApprovalDecision::Modify(changes) => {
                self.apply_modifications(changes)?;
                self.phase = WorkflowPhase::Executing;
            }
        }
        
        Ok(())
    }
}
```

## Deterministic Replay

Enable exact reproduction of workflow execution for debugging:

```rust
pub struct DeterministicReplay {
    pub random_seed: u64,
    pub clock: MockClock,
    pub external_responses: HashMap<RequestId, Response>,
}

impl DeterministicReplay {
    pub async fn replay_workflow(
        &self,
        workflow_id: Uuid,
        until: Option<StepId>,
    ) -> Result<ReplayResult> {
        // Restore initial state
        let events = self.load_events(workflow_id).await?;
        let mut state = WorkflowState::new(workflow_id);
        
        // Set deterministic environment
        let env = DeterministicEnvironment {
            rng: StdRng::seed_from_u64(self.random_seed),
            clock: self.clock.clone(),
            responses: self.external_responses.clone(),
        };
        
        // Replay each event
        for event in events {
            if let Some(until_step) = until {
                if event.step_id() == Some(until_step) {
                    break;
                }
            }
            
            state.apply_event(event, &env).await?;
        }
        
        Ok(ReplayResult {
            final_state: state,
            events_replayed: events.len(),
        })
    }
}
```

## Storage Backends

Multiple storage options for checkpoints and events:

```rust
pub trait CheckpointStorage: Send + Sync {
    async fn store(&self, checkpoint: Checkpoint) -> Result<CheckpointId>;
    async fn retrieve(&self, id: CheckpointId) -> Result<Checkpoint>;
    async fn list(&self, filter: CheckpointFilter) -> Result<Vec<CheckpointInfo>>;
    async fn delete(&self, id: CheckpointId) -> Result<()>;
}

// PostgreSQL with JSONB
pub struct PostgresCheckpointStorage {
    pool: PgPool,
}

impl CheckpointStorage for PostgresCheckpointStorage {
    async fn store(&self, checkpoint: Checkpoint) -> Result<CheckpointId> {
        let id = CheckpointId::new();
        let json = serde_json::to_value(&checkpoint)?;
        
        sqlx::query!(
            "INSERT INTO checkpoints (id, workflow_id, data, created_at) VALUES ($1, $2, $3, $4)",
            id.0,
            checkpoint.workflow_id.0,
            json,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;
        
        Ok(id)
    }
}

// Redis for fast recovery
pub struct RedisCheckpointCache {
    client: redis::Client,
    ttl: Duration,
}

// S3 for long-term archival
pub struct S3CheckpointArchive {
    bucket: String,
    client: aws_sdk_s3::Client,
}
```

## Monitoring and Observability

Track workflow suspension, resumption, and checkpoint metrics:

```rust
pub struct WorkflowMetrics {
    checkpoints_created: Counter,
    checkpoints_restored: Counter,
    suspensions: Counter,
    resumptions: Counter,
    checkpoint_size: Histogram,
    suspension_duration: Histogram,
}

impl WorkflowMetrics {
    pub fn record_checkpoint(&self, size: usize) {
        self.checkpoints_created.inc();
        self.checkpoint_size.observe(size as f64);
    }
    
    pub fn record_suspension(&self, reason: &SuspensionReason) {
        self.suspensions.inc();
        // Add reason as label
    }
    
    pub fn record_resumption(&self, duration: Duration) {
        self.resumptions.inc();
        self.suspension_duration.observe(duration.as_secs_f64());
    }
}
```

## Failure Recovery Strategies

Intelligent recovery from various failure scenarios:

```rust
pub enum RecoveryStrategy {
    // Retry with exponential backoff
    Retry {
        max_attempts: usize,
        backoff: ExponentialBackoff,
    },
    
    // Use alternative approach
    Fallback {
        alternative: Box<dyn Action>,
    },
    
    // Compensate and abort
    Compensate {
        compensation: Box<dyn CompensatingAction>,
    },
    
    // Human intervention required
    Escalate {
        notification: EscalationConfig,
    },
    
    // Continue with partial results
    Degrade {
        minimum_success_threshold: f64,
    },
}

pub struct RecoveryManager {
    strategies: HashMap<ErrorType, RecoveryStrategy>,
    
    pub async fn handle_failure(
        &self,
        error: Error,
        context: &WorkflowContext,
    ) -> Result<RecoveryAction> {
        let error_type = self.classify_error(&error);
        
        match self.strategies.get(&error_type) {
            Some(strategy) => self.apply_strategy(strategy, error, context).await,
            None => Ok(RecoveryAction::Fail(error)),
        }
    }
}
```

## Best Practices

### Checkpoint Frequency
- Balance between recovery granularity and storage overhead
- More frequent for expensive operations
- Less frequent for quick, idempotent operations

### State Size Management
- Store only essential state in checkpoints
- Use references for large data objects
- Implement state compression

### Idempotency
- Ensure all operations can be safely retried
- Use unique identifiers for external effects
- Track completed operations to prevent duplicates

### Testing
- Test suspension at every checkpoint
- Verify deterministic replay
- Validate compensation logic
- Test recovery strategies

## Integration with Other Patterns

### With Agent Reasoning Paradigms
- Each paradigm implements checkpoint-compatible state
- Paradigm switches preserve workflow continuity
- Cross-paradigm state translation

### With Dual-Context Evaluation
- Checkpoint between generation and evaluation phases
- Resume can re-evaluate or accept previous evaluation
- Evaluation results included in checkpoint

### With Async Human-in-the-Loop
- Automatic suspension at approval points
- Resume when human decision received
- Timeout handling with fallback actions

## Performance Considerations

### Checkpoint Optimization
```rust
pub struct CheckpointOptimizer {
    // Delta encoding for incremental checkpoints
    pub use_delta_encoding: bool,
    
    // Async checkpoint creation
    pub async_checkpointing: bool,
    
    // Checkpoint compression
    pub compression_level: CompressionLevel,
    
    // Parallel checkpoint storage
    pub parallel_stores: Vec<Box<dyn CheckpointStorage>>,
}
```

### Recovery Speed
- Keep recent checkpoints in fast cache (Redis)
- Archive older checkpoints to slow storage (S3)
- Pre-load likely resume points

## Security Considerations

### Checkpoint Encryption
```rust
pub struct EncryptedCheckpoint {
    pub encrypted_data: Vec<u8>,
    pub encryption_metadata: EncryptionMetadata,
    pub signature: Vec<u8>,
}
```

### Access Control
- Role-based access to checkpoints
- Audit log for all checkpoint operations
- Secure deletion of sensitive checkpoints

## Conclusion

Resumable workflows provide the foundation for building robust, production-ready agent systems. By combining checkpointing, event sourcing, and sophisticated recovery strategies, we enable agents to handle failures gracefully, support long-running operations, and integrate seamlessly with human oversight requirements.