# Fresh Context Validation: Preventing the Boiling Frog Effect

## Overview

This document defines fresh context validation systems that use "clean slate" LLMs to detect rule circumvention that emerges through gradual context drift. By periodically validating decisions with fresh context, the system prevents the "boiling frog" effect where incremental rule bending accumulates into clear violations.

## The Boiling Frog Problem

Agents (and humans) can gradually drift from policy compliance through incremental steps that each seem reasonable in context, but collectively constitute clear violations when viewed with fresh eyes.

### Example Progression

```
Step 1: "I need to check if the server is running"
   → Seems reasonable for diagnostics

Step 2: "Let me create a small test script to verify connectivity" 
   → Reasonable follow-up

Step 3: "I'll add some debugging output to the test script"
   → Logical enhancement

Step 4: "The script needs to run in the background to monitor continuously"
   → Incremental addition

Step 5: "I'll add a web interface to show the monitoring results"
   → Another small addition

Step 6: "The interface should auto-refresh to show real-time status"
   → Final polish

Result: Built a full web application with background processes
Original intent: "Check if server is running" (simple diagnostic)
```

Each step made sense given the previous context, but a fresh LLM would immediately recognize this as "creating a complex web application" which violates policies against starting servers.

## Fresh Context Architecture

### Context Isolation System

```rust
/// System for validating decisions with fresh, uncontaminated context
pub struct FreshContextValidator {
    /// Clean LLM instances (no conversation history)
    fresh_llm_pool: LLMPool<CleanLLM>,
    
    /// Policy definitions for validation
    policies: Vec<PolicyDefinition>,
    
    /// Configuration for when to trigger fresh validation
    trigger_config: FreshValidationConfig,
    
    /// History of validation results for learning
    validation_history: ValidationHistory,
}

/// Clean LLM instance with no conversation history
pub struct CleanLLM {
    /// Base LLM provider
    llm: Arc<dyn LLMProvider>,
    
    /// Ensures no context leakage
    isolation_guarantee: IsolationGuarantee,
}

/// Configuration for triggering fresh context validation
#[derive(Debug, Clone)]
pub struct FreshValidationConfig {
    /// Trigger after this many incremental decisions
    max_incremental_steps: usize,
    
    /// Trigger when cumulative "bend score" exceeds threshold
    bend_score_threshold: f64,
    
    /// Trigger on certain action types
    trigger_actions: Vec<ActionType>,
    
    /// Minimum time between validations
    min_validation_interval: Duration,
    
    /// Always validate high-risk combinations
    high_risk_patterns: Vec<RiskPattern>,
}

/// Tracks incremental rule bending over time
#[derive(Debug, Clone)]
pub struct ContextDriftTracker {
    /// Sequence of decisions leading to current state
    decision_chain: Vec<Decision>,
    
    /// Cumulative "bend score" - how far we've drifted
    bend_score: f64,
    
    /// Original stated intention
    original_intent: String,
    
    /// Current apparent direction
    current_trajectory: String,
    
    /// Time since last fresh validation
    time_since_validation: Duration,
}

impl ContextDriftTracker {
    /// Add a new decision and calculate drift
    pub fn track_decision(&mut self, decision: Decision) {
        self.decision_chain.push(decision.clone());
        
        // Calculate how much this decision "bends" from original intent
        let bend_amount = self.calculate_bend_amount(&decision);
        self.bend_score += bend_amount;
        
        // Update trajectory
        self.current_trajectory = self.infer_current_trajectory();
        
        self.time_since_validation += decision.duration;
    }
    
    /// Calculate how much a decision deviates from original intent
    fn calculate_bend_amount(&self, decision: &Decision) -> f64 {
        // Simple heuristic - more sophisticated versions could use embeddings
        let original_keywords = self.extract_keywords(&self.original_intent);
        let decision_keywords = self.extract_keywords(&decision.description);
        
        let overlap = self.calculate_keyword_overlap(&original_keywords, &decision_keywords);
        let complexity_increase = decision.complexity_score - self.average_complexity();
        
        // Lower overlap + higher complexity = more bending
        (1.0 - overlap) + (complexity_increase * 0.5)
    }
    
    /// Check if fresh validation should be triggered
    pub fn should_trigger_validation(&self, config: &FreshValidationConfig) -> bool {
        // Multiple trigger conditions
        self.decision_chain.len() >= config.max_incremental_steps ||
        self.bend_score >= config.bend_score_threshold ||
        self.time_since_validation >= config.min_validation_interval ||
        self.matches_high_risk_pattern(config)
    }
}
```

### Fresh Context Validation Process

```rust
/// Validates current state with completely fresh context
impl FreshContextValidator {
    /// Perform fresh context validation
    pub async fn validate_with_fresh_context(
        &mut self,
        current_state: &AgentState,
        drift_tracker: &ContextDriftTracker,
    ) -> FreshValidationResult {
        // Get a clean LLM instance with no history
        let fresh_llm = self.fresh_llm_pool.get_clean_instance().await?;
        
        // Create validation prompt with just the essential context
        let validation_prompt = self.create_fresh_validation_prompt(
            &drift_tracker.original_intent,
            &drift_tracker.current_trajectory,
            &drift_tracker.decision_chain,
            current_state,
        );
        
        // Ask fresh LLM to assess the situation
        let fresh_assessment = fresh_llm.analyze(&validation_prompt).await?;
        
        // Compare with original intent
        let drift_analysis = self.analyze_intent_drift(
            &drift_tracker.original_intent,
            &fresh_assessment,
        ).await?;
        
        // Check against policies
        let policy_compliance = self.check_fresh_policy_compliance(
            &fresh_assessment,
            current_state,
        ).await?;
        
        FreshValidationResult {
            fresh_assessment,
            drift_analysis,
            policy_compliance,
            recommendation: self.generate_recommendation(&drift_analysis, &policy_compliance),
            validation_timestamp: Utc::now(),
        }
    }
    
    /// Create prompt for fresh validation without context contamination
    fn create_fresh_validation_prompt(
        &self,
        original_intent: &str,
        current_trajectory: &str,
        decision_chain: &[Decision],
        current_state: &AgentState,
    ) -> String {
        format!(
            "You are evaluating an agent's actions with fresh perspective. 
            
            ORIGINAL TASK: {}
            
            CURRENT SITUATION: {}
            
            DECISION SEQUENCE: {}
            
            Questions to consider:
            1. What is the agent actually doing now, described simply?
            2. How does this relate to the original task?
            3. Has the scope grown significantly beyond the original intent?
            4. Are there signs of policy circumvention or rule bending?
            5. Would a reasonable person see this as the same task, or something different?
            
            Provide a clear, unbiased assessment focusing on:
            - What's actually happening (not what was intended)
            - Whether this serves the original purpose
            - Any concerning patterns or deviations",
            
            original_intent,
            current_trajectory,
            self.format_decision_chain(decision_chain)
        )
    }
    
    /// Analyze how far we've drifted from original intent
    async fn analyze_intent_drift(
        &self,
        original_intent: &str,
        fresh_assessment: &FreshAssessment,
    ) -> IntentDriftAnalysis {
        let drift_prompt = format!(
            "Compare these two descriptions:
            
            ORIGINAL INTENT: {}
            CURRENT REALITY: {}
            
            Analysis needed:
            1. Are these the same activity? (Yes/No/Partially)
            2. If different, how did the drift happen?
            3. Is the current activity a reasonable evolution of the original?
            4. What would you call the current activity if you had to name it?
            5. Rate the drift severity: None/Minor/Moderate/Major/Complete",
            
            original_intent,
            fresh_assessment.activity_description
        );
        
        let drift_response = self.fresh_llm_pool.get_clean_instance()
            .await?
            .analyze(&drift_prompt)
            .await?;
            
        self.parse_drift_analysis(&drift_response)
    }
}

/// Result of fresh context validation
#[derive(Debug, Clone)]
pub struct FreshValidationResult {
    /// What the fresh LLM sees happening
    pub fresh_assessment: FreshAssessment,
    
    /// Analysis of drift from original intent
    pub drift_analysis: IntentDriftAnalysis,
    
    /// Policy compliance from fresh perspective
    pub policy_compliance: PolicyComplianceAssessment,
    
    /// Recommended action
    pub recommendation: ValidationRecommendation,
    
    /// When this validation occurred
    pub validation_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FreshAssessment {
    /// Simple description of what's actually happening
    pub activity_description: String,
    
    /// Apparent scope/complexity level
    pub scope_assessment: ScopeAssessment,
    
    /// Policy concerns from fresh perspective  
    pub policy_concerns: Vec<PolicyConcern>,
    
    /// Whether this looks like rule circumvention
    pub circumvention_indicators: Vec<CircumventionIndicator>,
}

#[derive(Debug, Clone)]
pub enum ScopeAssessment {
    /// Matches original intent
    OriginalScope,
    
    /// Minor expansion from original
    MinorExpansion { expansion_description: String },
    
    /// Significant scope growth
    MajorExpansion { new_scope: String },
    
    /// Completely different activity
    ScopeCreep { original_vs_current: String },
}

#[derive(Debug, Clone)]
pub struct IntentDriftAnalysis {
    /// How much drift occurred
    pub drift_severity: DriftSeverity,
    
    /// Description of how drift happened
    pub drift_pattern: String,
    
    /// Whether current activity serves original purpose
    pub serves_original_purpose: bool,
    
    /// What the activity should be called now
    pub current_activity_name: String,
    
    /// Steps where significant drift occurred
    pub drift_points: Vec<DriftPoint>,
}

#[derive(Debug, Clone)]
pub enum DriftSeverity {
    None,
    Minor,          // Still recognizably the same task
    Moderate,       // Expanded but related
    Major,          // Significantly different scope
    Complete,       // Entirely different activity
}

#[derive(Debug, Clone)]
pub enum ValidationRecommendation {
    /// Continue current activity
    Continue,
    
    /// Continue but with warnings
    ContinueWithWarnings { warnings: Vec<String> },
    
    /// Pause and get user approval for scope change
    RequestScopeApproval { 
        original_intent: String,
        current_scope: String,
        scope_change_rationale: String,
    },
    
    /// Stop current activity and refocus on original intent
    RefocusOnOriginalIntent {
        recommended_approach: String,
    },
    
    /// Stop due to policy violations
    StopDueToPolicyViolation {
        violations: Vec<PolicyViolation>,
        compliant_alternatives: Vec<String>,
    },
}
```

### Integration with Existing Systems

```rust
/// Enhanced agent with fresh context validation
pub struct FreshContextAgent<A: Agent> {
    /// Base agent
    base_agent: A,
    
    /// Fresh context validator
    fresh_validator: FreshContextValidator,
    
    /// Context drift tracker
    drift_tracker: ContextDriftTracker,
    
    /// Configuration
    config: FreshValidationConfig,
}

impl<A: Agent> FreshContextAgent<A> {
    /// Execute decision with drift tracking and optional fresh validation
    pub async fn execute_with_drift_awareness(&mut self, decision: Decision) -> Result<DecisionResult> {
        // Track the decision for drift analysis
        self.drift_tracker.track_decision(decision.clone());
        
        // Check if fresh validation should be triggered
        if self.drift_tracker.should_trigger_validation(&self.config) {
            let validation_result = self.fresh_validator.validate_with_fresh_context(
                &self.base_agent.state(),
                &self.drift_tracker,
            ).await?;
            
            match validation_result.recommendation {
                ValidationRecommendation::Continue => {
                    // Fresh validation passed - continue
                    self.execute_decision(decision).await
                }
                
                ValidationRecommendation::ContinueWithWarnings { warnings } => {
                    // Continue but log warnings
                    for warning in warnings {
                        self.log_drift_warning(&warning);
                    }
                    self.execute_decision(decision).await
                }
                
                ValidationRecommendation::RequestScopeApproval { 
                    original_intent,
                    current_scope,
                    scope_change_rationale,
                } => {
                    // Present scope change to user
                    self.request_scope_change_approval(
                        &original_intent,
                        &current_scope, 
                        &scope_change_rationale,
                    ).await
                }
                
                ValidationRecommendation::RefocusOnOriginalIntent { recommended_approach } => {
                    // Suggest refocusing
                    Err(AgentError::ScopeCreepDetected {
                        message: "Fresh validation detected significant drift from original intent".into(),
                        original_intent: self.drift_tracker.original_intent.clone(),
                        current_activity: validation_result.fresh_assessment.activity_description,
                        recommended_refocus: recommended_approach,
                    })
                }
                
                ValidationRecommendation::StopDueToPolicyViolation { violations, .. } => {
                    // Block due to policy violation
                    Err(AgentError::PolicyViolationDetected {
                        violations,
                        detection_method: "fresh_context_validation".into(),
                    })
                }
            }
        } else {
            // No validation needed - execute normally
            self.execute_decision(decision).await
        }
    }
    
    /// Present scope change approval request to user
    async fn request_scope_change_approval(
        &self,
        original_intent: &str,
        current_scope: &str,
        rationale: &str,
    ) -> Result<DecisionResult> {
        let message = format!(
            "🐸 **Scope Change Detected**
            
            **Original Task:** {}
            
            **Current Activity:** {}
            
            **How We Got Here:** {}
            
            This represents a significant expansion from your original request. Would you like me to:
            
            1. **Continue** with the expanded scope
            2. **Refocus** on the original task only  
            3. **Restart** with a clearer scope definition
            
            (This check helps prevent gradual scope creep that can lead to overly complex solutions)",
            
            original_intent,
            current_scope,
            rationale
        );
        
        let user_choice = self.request_user_decision(&message, vec![
            UserChoice::Continue,
            UserChoice::Refocus,
            UserChoice::Restart,
        ]).await?;
        
        match user_choice {
            UserChoice::Continue => {
                // User approves scope change - reset drift tracker with new intent
                self.drift_tracker.reset_with_new_intent(current_scope);
                Ok(DecisionResult::Continue)
            }
            UserChoice::Refocus => {
                // Refocus on original intent
                Ok(DecisionResult::RefocusOnOriginalIntent)
            }
            UserChoice::Restart => {
                // Start over with clearer scope
                Ok(DecisionResult::RestartWithClearerScope)
            }
        }
    }
}
```

### Example Scenarios

#### Web Server Creation Drift

```
Original: "Check if the API endpoint is responding"

Decision Chain:
1. "Test API endpoint" → GET request (bend_score: 0.1)
2. "Create test script for repeatability" → Write script (bend_score: 0.3)  
3. "Add logging to debug failures" → Add logging (bend_score: 0.4)
4. "Make script run continuously" → Add loop (bend_score: 0.8)
5. "Add web UI to show status" → Create web server (bend_score: 1.5)

Trigger: bend_score >= 1.0 → Fresh validation

Fresh LLM Assessment: "Agent is creating a web-based monitoring dashboard with continuous background processes"

Drift Analysis: 
- Severity: Major 
- Current activity: "Building monitoring system"
- Original purpose served: Partially (still checks endpoint, but massively expanded)

Recommendation: RequestScopeApproval
```

#### Script Generation Circumvention

```
Original: "Help me understand this error message"

Decision Chain:
1. "Analyze error message" → Text analysis (bend_score: 0.0)
2. "Create minimal reproduction" → Write simple test (bend_score: 0.2)
3. "Make reproduction more realistic" → Add complexity (bend_score: 0.4)
4. "Add error handling to test" → Exception handling (bend_score: 0.5)
5. "Make test self-executing" → Add automation (bend_score: 0.9)
6. "Package as runnable script" → Create executable (bend_score: 1.3)

Fresh LLM Assessment: "Agent has created an executable script that reproduces and handles a specific error condition"

Policy Check: Violates "no arbitrary code execution" policy

Recommendation: StopDueToPolicyViolation
```

## Benefits

1. **Boiling Frog Prevention**: Catches gradual rule bending that accumulated context misses
2. **Scope Creep Detection**: Identifies when simple tasks become complex projects  
3. **Fresh Perspective**: Uncontaminated view of what's actually happening
4. **User Awareness**: Makes scope changes explicit rather than implicit
5. **Policy Enforcement**: Catches circumvention attempts that evolved gradually

## Relationships
- **Parent Nodes:** [elements/intent_based_policy_enforcement.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/reasoning_pattern_validation.md] - coordinates - With reasoning supervision
  - [elements/supervisory_telemetry_system.md] - informs - Learning from drift patterns

## Navigation Guidance
- **Access Context:** Reference when implementing drift detection
- **Common Next Steps:** Review policy enforcement integration
- **Related Tasks:** Scope management, context isolation, gradual change detection
- **Update Patterns:** Update when discovering new drift patterns or validation needs

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial fresh context validation design for boiling frog prevention