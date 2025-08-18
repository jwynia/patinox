# Asynchronous Human-in-the-Loop Patterns

## Overview

Traditional human-in-the-loop (HITL) systems assume the human is actively engaged at a console, ready to provide immediate feedback. However, real-world scenarios often require asynchronous human involvement - where humans provide oversight, approval, and guidance without blocking agent execution. This document explores patterns for integrating human judgment into agent workflows when the human isn't directly present.

## Core Challenges

The async HITL pattern must address several key challenges:

1. **Latency**: Humans may not respond for hours or days
2. **Context Loss**: Humans need sufficient context when they finally engage
3. **Continuation**: Agents must gracefully pause and resume
4. **Escalation**: Critical decisions may need immediate attention
5. **Fallback**: Systems need defaults when humans don't respond

## Architectural Patterns

### Approval Queue System

```rust
pub struct ApprovalQueue {
    // Pending approvals organized by priority
    pending: PriorityQueue<ApprovalRequest>,
    
    // Active approvals being reviewed
    active: HashMap<ApprovalId, ActiveApproval>,
    
    // Completed approvals with decisions
    completed: HashMap<ApprovalId, ApprovalDecision>,
    
    // Notification system
    notifier: Box<dyn NotificationService>,
}

pub struct ApprovalRequest {
    pub id: ApprovalId,
    pub workflow_id: WorkflowId,
    pub request_type: ApprovalType,
    pub context: ApprovalContext,
    pub priority: Priority,
    pub deadline: Option<DateTime<Utc>>,
    pub fallback_action: FallbackAction,
    pub requested_at: DateTime<Utc>,
    pub requested_by: AgentId,
}

pub struct ApprovalContext {
    // What the agent was doing
    pub task_description: String,
    
    // Why approval is needed
    pub approval_reason: String,
    
    // Relevant data for decision
    pub decision_data: serde_json::Value,
    
    // Options available to approver
    pub available_actions: Vec<ApprovalAction>,
    
    // Consequences of each action
    pub impact_analysis: HashMap<ApprovalAction, Impact>,
    
    // Historical context
    pub previous_decisions: Vec<RelatedDecision>,
}
```

### Notification and Escalation

```rust
pub struct NotificationStrategy {
    pub channels: Vec<NotificationChannel>,
    pub escalation_ladder: EscalationLadder,
    pub retry_policy: RetryPolicy,
}

pub enum NotificationChannel {
    Email {
        recipients: Vec<EmailAddress>,
        template: EmailTemplate,
    },
    Slack {
        channel: String,
        mention_users: Vec<UserId>,
    },
    Webhook {
        url: Url,
        headers: HashMap<String, String>,
    },
    SMS {
        numbers: Vec<PhoneNumber>,
        urgent_only: bool,
    },
    Dashboard {
        update_frequency: Duration,
    },
}

pub struct EscalationLadder {
    pub levels: Vec<EscalationLevel>,
}

pub struct EscalationLevel {
    pub timeout: Duration,
    pub approvers: Vec<Approver>,
    pub notification_channels: Vec<NotificationChannel>,
    pub escalation_message: String,
}

impl ApprovalQueue {
    pub async fn escalate_if_needed(&mut self) -> Result<()> {
        let now = Utc::now();
        
        for (id, request) in self.pending.iter() {
            let age = now - request.requested_at;
            
            if let Some(level) = self.get_escalation_level(age, &request) {
                self.notify_escalation(request, level).await?;
                
                if level.is_final() && request.deadline.map_or(false, |d| now > d) {
                    self.apply_fallback_action(request).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

### Delegation Patterns

Humans can define rules for automatic approval:

```rust
pub struct DelegationRule {
    pub id: RuleId,
    pub created_by: UserId,
    pub conditions: Vec<Condition>,
    pub action: DelegatedAction,
    pub valid_until: Option<DateTime<Utc>>,
    pub max_uses: Option<usize>,
}

pub enum Condition {
    // Value-based conditions
    LessThan { field: String, value: f64 },
    GreaterThan { field: String, value: f64 },
    Equals { field: String, value: serde_json::Value },
    Contains { field: String, substring: String },
    
    // Risk-based conditions
    RiskScore { max: f64 },
    ConfidenceScore { min: f64 },
    
    // Context-based conditions
    TaskType { allowed_types: Vec<String> },
    TimeWindow { start: Time, end: Time },
    
    // Compound conditions
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),
}

pub struct DelegationEngine {
    rules: Vec<DelegationRule>,
    
    pub async fn check_delegation(&self, request: &ApprovalRequest) -> Option<DelegatedAction> {
        for rule in &self.rules {
            if self.matches_all_conditions(&rule.conditions, request) {
                if self.is_rule_valid(rule) {
                    return Some(rule.action.clone());
                }
            }
        }
        None
    }
}
```

### Context Preservation

Maintaining context for when humans eventually review:

```rust
pub struct ContextPreserver {
    pub snapshot_strategy: SnapshotStrategy,
    pub compression: CompressionConfig,
    pub retention_policy: RetentionPolicy,
}

pub enum SnapshotStrategy {
    // Capture everything
    Full {
        include_history: bool,
        include_logs: bool,
    },
    
    // Capture only relevant parts
    Selective {
        selectors: Vec<ContextSelector>,
    },
    
    // Progressive detail levels
    Hierarchical {
        summary_level: SummaryConfig,
        detail_level: DetailConfig,
        full_level: FullConfig,
    },
}

impl ContextPreserver {
    pub async fn preserve_context(&self, workflow: &Workflow) -> Result<PreservedContext> {
        let raw_context = match self.snapshot_strategy {
            SnapshotStrategy::Full { include_history, include_logs } => {
                self.capture_full_context(workflow, include_history, include_logs).await?
            }
            SnapshotStrategy::Selective { ref selectors } => {
                self.capture_selective_context(workflow, selectors).await?
            }
            SnapshotStrategy::Hierarchical { .. } => {
                self.capture_hierarchical_context(workflow).await?
            }
        };
        
        let compressed = self.compress_context(raw_context)?;
        
        Ok(PreservedContext {
            workflow_id: workflow.id,
            timestamp: Utc::now(),
            data: compressed,
            expiry: self.calculate_expiry(),
        })
    }
}
```

### Asynchronous Decision Integration

```rust
pub struct DecisionIntegrator {
    pub integration_strategy: IntegrationStrategy,
    pub validation: DecisionValidator,
}

pub enum IntegrationStrategy {
    // Resume immediately with decision
    Immediate,
    
    // Validate decision is still applicable
    ValidatedResumption {
        validator: Box<dyn ContextValidator>,
    },
    
    // Re-evaluate with human input as guidance
    GuidedRecomputation {
        weight: f64,
    },
    
    // Start fresh with human decision as constraint
    FreshStartWithConstraints,
}

impl DecisionIntegrator {
    pub async fn integrate_decision(
        &self,
        workflow: &mut Workflow,
        decision: ApprovalDecision,
        time_elapsed: Duration,
    ) -> Result<()> {
        // Check if context has changed significantly
        let context_drift = self.measure_context_drift(workflow, time_elapsed)?;
        
        match self.integration_strategy {
            IntegrationStrategy::Immediate => {
                workflow.apply_decision(decision)?;
            }
            IntegrationStrategy::ValidatedResumption { ref validator } => {
                if validator.is_still_valid(&workflow.context, &decision, context_drift)? {
                    workflow.apply_decision(decision)?;
                } else {
                    workflow.request_new_approval()?;
                }
            }
            IntegrationStrategy::GuidedRecomputation { weight } => {
                workflow.recompute_with_guidance(decision, weight)?;
            }
            IntegrationStrategy::FreshStartWithConstraints => {
                workflow.restart_with_constraints(decision.to_constraints())?;
            }
        }
        
        Ok(())
    }
}
```

## Approval Patterns

### Multi-Stage Approval

```rust
pub struct MultiStageApproval {
    pub stages: Vec<ApprovalStage>,
    pub aggregation: AggregationStrategy,
}

pub struct ApprovalStage {
    pub name: String,
    pub approvers: Vec<Approver>,
    pub required_approvals: usize,
    pub timeout: Duration,
    pub can_proceed_on_partial: bool,
}

pub enum AggregationStrategy {
    // All stages must approve
    Unanimous,
    
    // Majority of stages
    Majority,
    
    // Weighted voting
    Weighted(HashMap<StageId, f64>),
    
    // Custom logic
    Custom(Box<dyn AggregationLogic>),
}
```

### Conditional Approval Flows

```rust
pub struct ConditionalApprovalFlow {
    pub initial_check: ApprovalRequest,
    pub branches: Vec<ConditionalBranch>,
}

pub struct ConditionalBranch {
    pub condition: BranchCondition,
    pub approval_required: Option<ApprovalRequest>,
    pub auto_action: Option<AutoAction>,
}

impl ConditionalApprovalFlow {
    pub async fn execute(&self, context: &Context) -> Result<FlowResult> {
        let initial_decision = self.request_approval(&self.initial_check).await?;
        
        for branch in &self.branches {
            if branch.condition.matches(&initial_decision, context) {
                if let Some(ref approval) = branch.approval_required {
                    return self.request_approval(approval).await;
                } else if let Some(ref action) = branch.auto_action {
                    return Ok(FlowResult::AutoAction(action.clone()));
                }
            }
        }
        
        Ok(FlowResult::Default)
    }
}
```

## Fallback Strategies

### Time-Based Fallbacks

```rust
pub enum FallbackAction {
    // Proceed with default
    UseDefault(serde_json::Value),
    
    // Abort the workflow
    Abort {
        reason: String,
        cleanup: Option<CleanupAction>,
    },
    
    // Proceed with reduced scope
    Degrade {
        reduced_functionality: DegradedMode,
    },
    
    // Delegate to another system
    Delegate {
        target: DelegationTarget,
    },
    
    // Retry with different parameters
    RetryWithModification {
        modifications: Vec<Modification>,
    },
}

pub struct TimeBasedFallback {
    pub timeout: Duration,
    pub warning_threshold: Duration,
    pub action: FallbackAction,
}

impl TimeBasedFallback {
    pub async fn monitor(&self, request: &ApprovalRequest) -> Result<()> {
        let elapsed = Utc::now() - request.requested_at;
        
        if elapsed > self.timeout {
            self.execute_fallback(request).await?;
        } else if elapsed > self.warning_threshold {
            self.send_warning(request).await?;
        }
        
        Ok(())
    }
}
```

### Confidence-Based Auto-Approval

```rust
pub struct ConfidenceBasedApproval {
    pub confidence_threshold: f64,
    pub risk_threshold: f64,
    pub require_human_above_risk: f64,
}

impl ConfidenceBasedApproval {
    pub async fn evaluate(&self, request: &ApprovalRequest) -> ApprovalResult {
        let confidence = self.calculate_confidence(request)?;
        let risk = self.calculate_risk(request)?;
        
        if risk > self.require_human_above_risk {
            return ApprovalResult::RequireHuman;
        }
        
        if confidence > self.confidence_threshold && risk < self.risk_threshold {
            ApprovalResult::AutoApprove
        } else {
            ApprovalResult::RequireHuman
        }
    }
}
```

## Human Interface Design

### Approval Interfaces

```rust
pub struct ApprovalInterface {
    pub presentation: PresentationStrategy,
    pub interaction: InteractionMode,
    pub feedback: FeedbackMechanism,
}

pub enum PresentationStrategy {
    // Simple yes/no
    Binary {
        question: String,
    },
    
    // Multiple choice
    MultipleChoice {
        options: Vec<Choice>,
    },
    
    // Detailed review
    DetailedReview {
        sections: Vec<ReviewSection>,
        editable_fields: Vec<String>,
    },
    
    // Comparative
    Comparative {
        alternatives: Vec<Alternative>,
    },
}

pub enum InteractionMode {
    // Click a button
    SimpleClick,
    
    // Provide reasoning
    WithJustification {
        min_length: usize,
    },
    
    // Modify and approve
    EditAndApprove {
        editable_fields: Vec<String>,
    },
    
    // Structured feedback
    StructuredFeedback {
        template: FeedbackTemplate,
    },
}
```

### Mobile and Notification-Based Approval

```rust
pub struct MobileApproval {
    pub simplified_context: SimplifiedContext,
    pub quick_actions: Vec<QuickAction>,
    pub deep_link: Option<Url>,
}

pub struct NotificationApproval {
    pub inline_actions: Vec<InlineAction>,
    pub expiry: Duration,
    pub secure_token: SecureToken,
}

impl NotificationApproval {
    pub async fn process_email_response(&self, email: Email) -> Result<ApprovalDecision> {
        // Parse approval from email reply
        let token = self.extract_token(&email)?;
        self.verify_token(token)?;
        
        let decision = self.parse_decision(&email.body)?;
        Ok(decision)
    }
    
    pub async fn process_slack_interaction(&self, interaction: SlackInteraction) -> Result<ApprovalDecision> {
        // Handle Slack button clicks or slash commands
        let decision = match interaction {
            SlackInteraction::Button { action_id, .. } => {
                self.map_action_to_decision(action_id)?
            }
            SlackInteraction::SlashCommand { text, .. } => {
                self.parse_command(text)?
            }
        };
        
        Ok(decision)
    }
}
```

## Audit and Compliance

### Decision Audit Trail

```rust
pub struct AuditTrail {
    pub entries: Vec<AuditEntry>,
    pub retention_policy: RetentionPolicy,
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub request: ApprovalRequest,
    pub decision: Option<ApprovalDecision>,
    pub decider: Decider,
    pub context_hash: Hash,
    pub justification: Option<String>,
}

pub enum Decider {
    Human(UserId),
    DelegationRule(RuleId),
    Fallback(FallbackAction),
    System(SystemComponent),
}

impl AuditTrail {
    pub fn validate_compliance(&self) -> ComplianceReport {
        let mut report = ComplianceReport::new();
        
        for requirement in &self.compliance_requirements {
            let validation = requirement.validate(&self.entries);
            report.add_validation(requirement.id(), validation);
        }
        
        report
    }
}
```

## Performance Optimization

### Batch Approval Processing

```rust
pub struct BatchApprovalProcessor {
    pub batching_strategy: BatchingStrategy,
    pub batch_size: usize,
    pub batch_timeout: Duration,
}

pub enum BatchingStrategy {
    // Group by approver
    ByApprover,
    
    // Group by type
    ByType,
    
    // Group by priority
    ByPriority,
    
    // Smart batching
    Adaptive {
        optimizer: Box<dyn BatchOptimizer>,
    },
}

impl BatchApprovalProcessor {
    pub async fn process_batch(&self, requests: Vec<ApprovalRequest>) -> Vec<ApprovalDecision> {
        let batches = self.create_batches(requests);
        let mut decisions = Vec::new();
        
        for batch in batches {
            let batch_decision = self.request_batch_approval(batch).await?;
            decisions.extend(batch_decision);
        }
        
        decisions
    }
}
```

### Predictive Pre-Approval

```rust
pub struct PredictiveApproval {
    pub predictor: Box<dyn ApprovalPredictor>,
    pub confidence_threshold: f64,
    pub pre_approval_cache: Cache<RequestPattern, ApprovalDecision>,
}

impl PredictiveApproval {
    pub async fn check_pre_approval(&self, request: &ApprovalRequest) -> Option<ApprovalDecision> {
        let pattern = self.extract_pattern(request);
        
        // Check cache first
        if let Some(cached) = self.pre_approval_cache.get(&pattern) {
            return Some(cached.clone());
        }
        
        // Try to predict
        let (prediction, confidence) = self.predictor.predict(request).await?;
        
        if confidence > self.confidence_threshold {
            self.pre_approval_cache.insert(pattern, prediction.clone());
            Some(prediction)
        } else {
            None
        }
    }
}
```

## Integration with Other Patterns

### With Resumable Workflows
- Workflows automatically suspend at approval points
- State is checkpointed before requesting approval
- Resume with decision integrated into context

### With Dual-Context Evaluation
- Evaluation context can trigger approval requests
- Human decisions override evaluation scores
- Feedback from humans improves evaluation criteria

### With Agent Reasoning Paradigms
- Different paradigms may have different approval thresholds
- ToT/GoT branches can be presented for human selection
- Human can specify which paradigm to use

## Best Practices

### Request Design
1. Provide sufficient context without overwhelming
2. Make the decision impact clear
3. Offer reasonable default options
4. Set appropriate timeouts based on criticality

### Notification Strategy
1. Use multiple channels for critical approvals
2. Implement smart notification suppression
3. Provide one-click approval when possible
4. Include context in notifications

### Fallback Design
1. Always have a safe fallback action
2. Make fallback consequences clear
3. Log when fallbacks are triggered
4. Review fallback patterns for improvement

### Security
1. Authenticate all approval decisions
2. Implement approval signing/verification
3. Audit all decisions and delegations
4. Encrypt sensitive context data

## Conclusion

Asynchronous human-in-the-loop patterns enable practical human oversight of agent systems without requiring constant human attention. By implementing approval queues, delegation rules, smart notifications, and intelligent fallbacks, we can maintain human control while allowing agents to operate efficiently. The key is balancing autonomy with oversight, ensuring humans are involved for critical decisions while not becoming bottlenecks for routine operations.