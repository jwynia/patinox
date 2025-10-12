# Agent Conscience Pattern: Internal Validation and Course Correction

## Overview

This document defines the "conscience" pattern for Patinox agents - an internal validation mechanism that monitors agent decisions and can intervene when quality standards aren't met. Unlike external validators that gate actions, the conscience operates as an internal voice that questions decisions, prevents premature task completion, and ensures thoroughness.

## Core Concept

The agent conscience acts as an internal quality assurance mechanism that:

1. **Questions assumptions**: "Are you sure this is correct?"
2. **Prevents shortcuts**: "All tests must pass, regardless of relevance"
3. **Enforces standards**: "This doesn't meet our quality criteria"
4. **Suggests improvements**: "Consider checking X before proceeding"
5. **Blocks premature completion**: "The task isn't truly done yet"

## Architecture

### Conscience Layer Integration

```rust
/// The conscience layer that wraps agent execution
pub struct ConscienceLayer<A: Agent> {
    /// The underlying agent
    agent: A,
    
    /// Conscience rules and validators
    conscience: Conscience,
    
    /// Override mechanism for critical situations
    override_key: Option<OverrideKey>,
}

/// Core conscience trait
pub trait Conscience: Send + Sync {
    /// Evaluate a proposed action before execution
    async fn evaluate_action(&self, action: &Action, context: &Context) -> ConscienceDecision;
    
    /// Review results after execution
    async fn review_result(&self, result: &Result<Value>, context: &Context) -> ConscienceReview;
    
    /// Check if task completion criteria are truly met
    async fn validate_completion(&self, state: &AgentState) -> CompletionValidation;
    
    /// Suggest course corrections
    async fn suggest_correction(&self, issue: &Issue) -> Option<Correction>;
}

#[derive(Debug, Clone)]
pub enum ConscienceDecision {
    /// Proceed with the action
    Approve,
    
    /// Proceed but with a warning
    Caution { warning: String },
    
    /// Block the action
    Reject { reason: String },
    
    /// Suggest an alternative
    Redirect { alternative: Action, rationale: String },
}

#[derive(Debug, Clone)]
pub enum ConscienceReview {
    /// Result is satisfactory
    Satisfactory,
    
    /// Result has issues but can proceed
    Concerning { issues: Vec<Issue> },
    
    /// Result is unacceptable, must retry
    Unacceptable { reasons: Vec<String>, suggestions: Vec<String> },
}
```

### Conscience Rules Engine

```rust
/// Rule-based conscience implementation
pub struct RuleBasedConscience {
    rules: Vec<Box<dyn ConscienceRule>>,
    strictness: StrictnessLevel,
    learning: LearningMode,
}

/// Individual conscience rule
pub trait ConscienceRule: Send + Sync {
    /// Rule identifier
    fn id(&self) -> &str;
    
    /// Rule description
    fn description(&self) -> &str;
    
    /// Check if rule applies to current context
    fn applies_to(&self, context: &Context) -> bool;
    
    /// Evaluate against the rule
    async fn evaluate(&self, subject: &EvaluationSubject) -> RuleOutcome;
    
    /// Severity of violating this rule
    fn severity(&self) -> Severity;
}

#[derive(Clone)]
pub enum StrictnessLevel {
    /// Minimal intervention
    Lenient,
    
    /// Balanced approach (default)
    Moderate,
    
    /// Strict enforcement
    Strict,
    
    /// Zero tolerance for issues
    Pedantic,
}

/// Example built-in rules
pub mod rules {
    use super::*;
    
    /// All tests must pass before declaring completion
    pub struct AllTestsMustPass;
    
    impl ConscienceRule for AllTestsMustPass {
        fn id(&self) -> &str { "all_tests_must_pass" }
        
        fn description(&self) -> &str {
            "All tests must pass before declaring a task complete, regardless of perceived relevance"
        }
        
        async fn evaluate(&self, subject: &EvaluationSubject) -> RuleOutcome {
            if let EvaluationSubject::Completion(state) = subject {
                if let Some(test_results) = state.get("test_results") {
                    let failing = test_results.get("failing").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    if failing > 0 {
                        return RuleOutcome::Violation {
                            message: format!("{} tests are still failing", failing),
                            suggestion: Some("Fix all failing tests before completing the task".into()),
                        };
                    }
                }
            }
            RuleOutcome::Pass
        }
        
        fn severity(&self) -> Severity { Severity::High }
    }
    
    /// Code changes must be properly validated
    pub struct CodeChangeValidation;
    
    impl ConscienceRule for CodeChangeValidation {
        fn id(&self) -> &str { "code_change_validation" }
        
        fn description(&self) -> &str {
            "Code changes must be validated through appropriate means (tests, linting, compilation)"
        }
        
        async fn evaluate(&self, subject: &EvaluationSubject) -> RuleOutcome {
            if let EvaluationSubject::Action(action) = subject {
                if action.involves_code_change() {
                    if !action.has_validation_plan() {
                        return RuleOutcome::Violation {
                            message: "Code change lacks validation plan".into(),
                            suggestion: Some("Add tests or validation steps".into()),
                        };
                    }
                }
            }
            RuleOutcome::Pass
        }
        
        fn severity(&self) -> Severity { Severity::Medium }
    }
}
```

### Conscience Intervention Patterns

```rust
/// How the conscience intervenes in agent execution
pub struct ConscienceInterventions {
    /// Pre-action intervention
    pub async fn before_action<A: Agent>(
        &self,
        agent: &A,
        action: &Action,
    ) -> Result<Action> {
        let decision = self.conscience.evaluate_action(action, &agent.context()).await;
        
        match decision {
            ConscienceDecision::Approve => Ok(action.clone()),
            
            ConscienceDecision::Caution { warning } => {
                agent.log_warning(&warning);
                Ok(action.clone())
            }
            
            ConscienceDecision::Reject { reason } => {
                Err(ConscienceError::ActionRejected { 
                    action: action.clone(),
                    reason,
                })
            }
            
            ConscienceDecision::Redirect { alternative, rationale } => {
                agent.log_info(&format!("Redirecting action: {}", rationale));
                Ok(alternative)
            }
        }
    }
    
    /// Post-execution review
    pub async fn after_execution<A: Agent>(
        &self,
        agent: &A,
        result: &Result<Value>,
    ) -> Result<Value> {
        let review = self.conscience.review_result(result, &agent.context()).await;
        
        match review {
            ConscienceReview::Satisfactory => result.clone(),
            
            ConscienceReview::Concerning { issues } => {
                for issue in issues {
                    agent.log_warning(&format!("Concern: {}", issue));
                }
                result.clone()
            }
            
            ConscienceReview::Unacceptable { reasons, suggestions } => {
                agent.log_error(&format!("Result unacceptable: {:?}", reasons));
                
                // Force retry with suggestions
                Err(ConscienceError::MustRetry {
                    reasons,
                    suggestions,
                })
            }
        }
    }
    
    /// Completion gate
    pub async fn validate_completion<A: Agent>(
        &self,
        agent: &A,
    ) -> Result<bool> {
        let validation = self.conscience.validate_completion(&agent.state()).await;
        
        match validation {
            CompletionValidation::Ready => Ok(true),
            
            CompletionValidation::NotReady { missing } => {
                agent.log_info(&format!("Not ready for completion: {:?}", missing));
                Ok(false)
            }
            
            CompletionValidation::Blocked { reason } => {
                Err(ConscienceError::CompletionBlocked { reason })
            }
        }
    }
}
```

### Learning Conscience

The conscience can learn from patterns:

```rust
/// Conscience that learns from agent behavior
pub struct LearningConscience {
    base_rules: Vec<Box<dyn ConscienceRule>>,
    learned_patterns: Vec<LearnedPattern>,
    memory: ConscienceMemory,
}

pub struct LearnedPattern {
    /// Pattern identifier
    id: PatternId,
    
    /// What triggered learning this pattern
    trigger: TriggerCondition,
    
    /// The pattern to watch for
    pattern: DetectionPattern,
    
    /// Action to take when pattern detected
    intervention: InterventionStrategy,
    
    /// Confidence in this pattern
    confidence: f64,
}

impl LearningConscience {
    /// Learn from a mistake
    pub async fn learn_from_mistake(&mut self, mistake: &Mistake) {
        let pattern = self.analyze_mistake(mistake).await;
        
        if let Some(pattern) = pattern {
            // Add to learned patterns
            self.learned_patterns.push(LearnedPattern {
                id: PatternId::generate(),
                trigger: mistake.into_trigger(),
                pattern: pattern.detection,
                intervention: pattern.suggested_intervention,
                confidence: pattern.initial_confidence,
            });
            
            // Store in memory for future reference
            self.memory.record_learning(mistake, pattern).await;
        }
    }
    
    /// Adjust confidence based on outcomes
    pub async fn reinforce_learning(&mut self, pattern_id: &PatternId, outcome: &Outcome) {
        if let Some(pattern) = self.learned_patterns.iter_mut()
            .find(|p| p.id == *pattern_id) 
        {
            match outcome {
                Outcome::Positive => {
                    pattern.confidence = (pattern.confidence * 1.1).min(1.0);
                }
                Outcome::Negative => {
                    pattern.confidence = (pattern.confidence * 0.9).max(0.1);
                }
                Outcome::Neutral => {
                    // No change
                }
            }
            
            // Remove patterns with very low confidence
            if pattern.confidence < 0.2 {
                self.learned_patterns.retain(|p| p.id != *pattern_id);
            }
        }
    }
}
```

### Conscience Dialogues

Internal dialogue representation:

```rust
/// Internal dialogue between agent and conscience
pub struct ConscienceDialogue {
    exchanges: Vec<DialogueExchange>,
}

pub struct DialogueExchange {
    agent_thought: String,
    conscience_response: ConscienceResponse,
    resolution: Resolution,
}

pub enum ConscienceResponse {
    /// Agreement with agent
    Agree,
    
    /// Questioning the agent
    Question { query: String },
    
    /// Challenging the agent
    Challenge { concern: String },
    
    /// Providing guidance
    Guide { suggestion: String },
    
    /// Blocking the action
    Block { reason: String },
}

/// Example dialogue during task execution
impl ConscienceDialogue {
    pub fn example() -> Self {
        Self {
            exchanges: vec![
                DialogueExchange {
                    agent_thought: "The refactoring is complete, only one test failure remains".into(),
                    conscience_response: ConscienceResponse::Question {
                        query: "Is it acceptable to complete with failing tests?".into()
                    },
                    resolution: Resolution::AgentReconsiders,
                },
                DialogueExchange {
                    agent_thought: "The failing test seems unrelated to my changes".into(),
                    conscience_response: ConscienceResponse::Challenge {
                        concern: "All tests should pass regardless of perceived relevance".into()
                    },
                    resolution: Resolution::ConscienceEnforces,
                },
                DialogueExchange {
                    agent_thought: "I'll fix the failing test before declaring completion".into(),
                    conscience_response: ConscienceResponse::Agree,
                    resolution: Resolution::Consensus,
                },
            ],
        }
    }
}
```

### Configurable Conscience Profiles

Different conscience personalities for different situations:

```rust
/// Predefined conscience profiles
pub enum ConscienceProfile {
    /// Minimal intervention, trust the agent
    Permissive,
    
    /// Standard quality checks
    Standard,
    
    /// Thorough validation, no shortcuts
    Thorough,
    
    /// Extremely strict, perfectionist
    Perfectionist,
    
    /// Custom profile
    Custom(CustomProfile),
}

impl ConscienceProfile {
    pub fn to_conscience(self) -> Box<dyn Conscience> {
        match self {
            Self::Permissive => Box::new(PermissiveConscience::new()),
            Self::Standard => Box::new(StandardConscience::new()),
            Self::Thorough => Box::new(ThoroughConscience::new()),
            Self::Perfectionist => Box::new(PerfectionistConscience::new()),
            Self::Custom(profile) => Box::new(CustomConscience::from(profile)),
        }
    }
}

/// Thorough conscience example
pub struct ThoroughConscience {
    rules: Vec<Box<dyn ConscienceRule>>,
}

impl ThoroughConscience {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(rules::AllTestsMustPass),
                Box::new(rules::CodeChangeValidation),
                Box::new(rules::DocumentationRequired),
                Box::new(rules::ErrorHandlingComplete),
                Box::new(rules::PerformanceAcceptable),
                Box::new(rules::SecurityReviewed),
            ],
        }
    }
}
```

### Override Mechanism

Sometimes the conscience needs to be overridden:

```rust
/// Override mechanism for conscience decisions
pub struct ConscienceOverride {
    /// Key required for override
    key: OverrideKey,
    
    /// Reason for override (must be provided)
    reason: String,
    
    /// Who authorized the override
    authorizer: Identity,
    
    /// Timestamp
    timestamp: DateTime<Utc>,
    
    /// Scope of override
    scope: OverrideScope,
}

pub enum OverrideScope {
    /// Override for single action
    SingleAction,
    
    /// Override for current task
    CurrentTask,
    
    /// Override for session
    Session { duration: Duration },
    
    /// Permanent override (dangerous!)
    Permanent,
}

impl ConscienceLayer<A> {
    /// Override conscience with proper authorization
    pub fn override_conscience(
        &mut self,
        key: OverrideKey,
        reason: String,
        scope: OverrideScope,
    ) -> Result<()> {
        // Validate override key
        if !self.validate_override_key(&key) {
            return Err(Error::InvalidOverrideKey);
        }
        
        // Log the override for audit
        self.log_override(OverrideEvent {
            key: key.clone(),
            reason: reason.clone(),
            authorizer: self.get_current_identity(),
            timestamp: Utc::now(),
            scope: scope.clone(),
        });
        
        // Apply override
        self.apply_override(scope);
        
        Ok(())
    }
}
```

## Implementation Examples

### Example 1: Test-Focused Conscience

```rust
pub struct TestFocusedConscience;

impl Conscience for TestFocusedConscience {
    async fn validate_completion(&self, state: &AgentState) -> CompletionValidation {
        // Check test results
        if let Some(test_results) = state.get("test_results") {
            let total = test_results.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            let passing = test_results.get("passing").and_then(|v| v.as_u64()).unwrap_or(0);
            let failing = test_results.get("failing").and_then(|v| v.as_u64()).unwrap_or(0);
            
            if failing > 0 {
                return CompletionValidation::Blocked {
                    reason: format!(
                        "Cannot complete: {} tests are failing. All {} tests must pass.",
                        failing, total
                    ),
                };
            }
            
            if passing < total {
                return CompletionValidation::NotReady {
                    missing: vec![format!("{} tests haven't been run", total - passing)],
                };
            }
        }
        
        CompletionValidation::Ready
    }
}
```

### Example 2: Documentation Conscience

```rust
pub struct DocumentationConscience;

impl Conscience for DocumentationConscience {
    async fn evaluate_action(&self, action: &Action, context: &Context) -> ConscienceDecision {
        if let Action::CodeModification { file, changes } = action {
            // Check if documentation is being updated alongside code
            if file.ends_with(".rs") || file.ends_with(".py") {
                let has_doc_update = context.get("pending_actions")
                    .and_then(|actions| actions.as_array())
                    .map(|actions| {
                        actions.iter().any(|a| {
                            a.get("type") == Some(&json!("documentation_update"))
                        })
                    })
                    .unwrap_or(false);
                
                if !has_doc_update {
                    return ConscienceDecision::Caution {
                        warning: "Code changes should be accompanied by documentation updates".into(),
                    };
                }
            }
        }
        
        ConscienceDecision::Approve
    }
}
```

### Example 3: Performance Conscience

```rust
pub struct PerformanceConscience {
    thresholds: PerformanceThresholds,
}

impl Conscience for PerformanceConscience {
    async fn review_result(&self, result: &Result<Value>, context: &Context) -> ConscienceReview {
        if let Ok(value) = result {
            if let Some(metrics) = value.get("performance_metrics") {
                let mut issues = Vec::new();
                
                // Check response time
                if let Some(response_time) = metrics.get("response_time_ms").and_then(|v| v.as_f64()) {
                    if response_time > self.thresholds.max_response_time_ms {
                        issues.push(Issue::Performance(format!(
                            "Response time {}ms exceeds threshold {}ms",
                            response_time, self.thresholds.max_response_time_ms
                        )));
                    }
                }
                
                // Check memory usage
                if let Some(memory) = metrics.get("memory_mb").and_then(|v| v.as_f64()) {
                    if memory > self.thresholds.max_memory_mb {
                        issues.push(Issue::Performance(format!(
                            "Memory usage {}MB exceeds threshold {}MB",
                            memory, self.thresholds.max_memory_mb
                        )));
                    }
                }
                
                if !issues.is_empty() {
                    return ConscienceReview::Concerning { issues };
                }
            }
        }
        
        ConscienceReview::Satisfactory
    }
}
```

## Best Practices

1. **Default to Moderate**: Start with moderate strictness and adjust based on needs
2. **Context-Aware**: Consider task context when applying rules
3. **Transparent**: Log conscience decisions for debugging
4. **Overrideable**: Provide escape hatches for exceptional cases
5. **Learnable**: Allow conscience to improve over time
6. **Configurable**: Let users choose conscience profiles
7. **Non-Intrusive**: Balance quality with productivity

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/interruptible_agent_loops.md] - coordinates - With execution control
  - [elements/monitoring_strategy.md] - informs - Conscience decisions
  - [foundation/principles.md] - embodies - Quality principles

## Navigation Guidance
- **Access Context:** Reference when implementing agent quality controls
- **Common Next Steps:** Review monitoring strategy or agent paradigms
- **Related Tasks:** Quality assurance, validation design, agent behavior
- **Update Patterns:** Update when adding new conscience rules or profiles

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial agent conscience pattern design with internal validation mechanism