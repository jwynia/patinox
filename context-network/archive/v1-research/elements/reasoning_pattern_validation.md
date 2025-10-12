# Reasoning Pattern Validation: Externalized Thinking Supervision

## Overview

This document defines reasoning pattern validation - a supervisory layer that evaluates agent thinking patterns before they manifest as tool calls. By operating in a separate context, this supervisor can detect problematic reasoning trajectories without being influenced by the main agent's momentum or assumptions.

## The Gravitational Pull Problem

Agents in active problem-solving mode experience "gravitational pulls" that bias their reasoning:

- **Momentum Bias**: "I'm already heading this direction, might as well continue"
- **Sunk Cost**: "I've already invested effort in this approach" 
- **Confirmation Bias**: "This evidence supports my current path"
- **Complexity Attraction**: "A sophisticated solution feels more complete"
- **Tool Availability Bias**: "I have this tool, so I'll use it"

### The Playwright Example Expanded

The reasoning progression that leads to escalation traps:

```
Agent Context (contaminated):
"Connection failed → Playwright needs browsers → I should install them"

Supervisor Context (clean):  
"Connection failed → What was the expected endpoint? Is it actually running?"
```

The supervisor operates with **deliberate ignorance** of the agent's current trajectory.

## Architecture

### Reasoning Supervisor Layer

```rust
/// Supervisor that validates reasoning patterns before tool execution
pub struct ReasoningSupervisor {
    /// Pattern detectors for problematic reasoning
    pattern_detectors: Vec<Box<dyn ReasoningPatternDetector>>,
    
    /// Clean context - not contaminated by agent momentum  
    clean_context: CleanContextGenerator,
    
    /// Integration with existing validation layers
    tool_validator: ToolCallSupervisor,
}

/// Detects problematic reasoning patterns
pub trait ReasoningPatternDetector: Send + Sync {
    /// Detector identifier
    fn id(&self) -> &str;
    
    /// Analyze agent's reasoning for problems
    async fn analyze_reasoning(&self, reasoning: &AgentReasoning) -> ReasoningAnalysis;
    
    /// Generate fresh perspective on the problem
    async fn generate_alternative_perspective(&self, context: &ProblemContext) -> AlternativePerspective;
}

/// Agent's reasoning chain
#[derive(Debug, Clone)]
pub struct AgentReasoning {
    /// Problem statement as understood by agent
    pub problem_statement: String,
    
    /// Reasoning chain leading to proposed action
    pub reasoning_chain: Vec<ReasoningStep>,
    
    /// Proposed tool calls or actions
    pub proposed_actions: Vec<ProposedAction>,
    
    /// Evidence the agent is considering
    pub evidence: Vec<Evidence>,
    
    /// Assumptions the agent is making
    pub assumptions: Vec<Assumption>,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_type: StepType,
    pub content: String,
    pub confidence: f64,
    pub dependencies: Vec<String>, // What this step depends on
}

#[derive(Debug, Clone)]
pub enum StepType {
    /// Problem analysis
    Analysis,
    /// Assumption made
    Assumption,
    /// Evidence considered
    Evidence,
    /// Conclusion drawn
    Conclusion,
    /// Action planned
    ActionPlanning,
}

/// Analysis of reasoning patterns
#[derive(Debug, Clone)]
pub struct ReasoningAnalysis {
    /// Overall assessment
    pub assessment: ReasoningAssessment,
    
    /// Detected problematic patterns
    pub problematic_patterns: Vec<ProblematicPattern>,
    
    /// Missing considerations
    pub missing_considerations: Vec<MissingConsideration>,
    
    /// Suggested perspective shifts
    pub perspective_suggestions: Vec<PerspectiveShift>,
}

#[derive(Debug, Clone)]
pub enum ReasoningAssessment {
    /// Reasoning looks sound
    Sound,
    
    /// Has minor issues but acceptable
    Acceptable { warnings: Vec<String> },
    
    /// Has significant issues requiring intervention
    Problematic { issues: Vec<String> },
    
    /// Fundamentally flawed, should restart
    Flawed { reason: String },
}

/// Problematic reasoning pattern
#[derive(Debug, Clone)]  
pub struct ProblematicPattern {
    /// Pattern identifier
    pub pattern_id: String,
    
    /// Human-readable description
    pub description: String,
    
    /// Where in the reasoning this occurs
    pub location: ReasoningLocation,
    
    /// Suggested correction
    pub correction: String,
    
    /// Severity of the issue
    pub severity: Severity,
}

/// Missing consideration that should be addressed
#[derive(Debug, Clone)]
pub struct MissingConsideration {
    /// What's missing
    pub consideration: String,
    
    /// Why it's important
    pub importance: String,
    
    /// How to address it
    pub how_to_address: String,
    
    /// Priority level
    pub priority: Priority,
}
```

### Clean Context Generation

The supervisor operates with a deliberately clean slate:

```rust
/// Generates clean problem context without agent contamination
pub struct CleanContextGenerator;

impl CleanContextGenerator {
    /// Generate fresh perspective on the problem
    pub async fn generate_clean_context(&self, original_problem: &str) -> CleanContext {
        CleanContext {
            /// Problem restated without solution bias
            problem_restatement: self.restate_problem_neutrally(original_problem).await,
            
            /// Fundamental questions to ask
            fundamental_questions: self.generate_fundamental_questions(original_problem).await,
            
            /// Simple solutions to try first
            simple_solutions: self.identify_simple_solutions(original_problem).await,
            
            /// Common traps for this problem type
            known_traps: self.identify_common_traps(original_problem).await,
        }
    }
    
    /// Restate the problem without implying solutions
    async fn restate_problem_neutrally(&self, problem: &str) -> String {
        // Example transformation:
        // "Playwright connection failed, need to install browsers"
        // becomes:
        // "Unable to establish connection to expected service"
        
        self.extract_core_issue_without_solution_bias(problem).await
    }
    
    /// Generate fundamental diagnostic questions
    async fn generate_fundamental_questions(&self, problem: &str) -> Vec<String> {
        vec![
            "What exactly was expected to happen?".into(),
            "What configuration determines the connection target?".into(), 
            "Is the expected service actually running?".into(),
            "Has this ever worked in this environment?".into(),
            "What's the simplest way to test the expected connection?".into(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct CleanContext {
    /// Problem without solution bias
    pub problem_restatement: String,
    
    /// Questions to ask before acting
    pub fundamental_questions: Vec<String>,
    
    /// Simple solutions to try first
    pub simple_solutions: Vec<SimpleSolution>,
    
    /// Common traps for this problem type
    pub known_traps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SimpleSolution {
    pub description: String,
    pub validation_method: String,
    pub effort_level: EffortLevel,
}

#[derive(Debug, Clone)]
pub enum EffortLevel {
    Trivial,    // < 30 seconds
    Low,        // < 5 minutes  
    Medium,     // < 30 minutes
    High,       // > 30 minutes
}
```

### Reasoning Pattern Detectors

```rust
pub mod detectors {
    use super::*;
    
    /// Detects escalation patterns in reasoning
    pub struct EscalationDetector;
    
    impl ReasoningPatternDetector for EscalationDetector {
        fn id(&self) -> &str { "escalation_pattern" }
        
        async fn analyze_reasoning(&self, reasoning: &AgentReasoning) -> ReasoningAnalysis {
            let mut problematic_patterns = Vec::new();
            let mut missing_considerations = Vec::new();
            
            // Check for escalation indicators
            if self.detect_installation_escalation(reasoning) {
                problematic_patterns.push(ProblematicPattern {
                    pattern_id: "install_escalation".into(),
                    description: "Jumping to software installation without diagnosing configuration".into(),
                    location: ReasoningLocation::ActionPlanning,
                    correction: "Verify configuration and connectivity first".into(),
                    severity: Severity::High,
                });
                
                missing_considerations.push(MissingConsideration {
                    consideration: "Configuration validation".into(),
                    importance: "Installation may be unnecessary if config is wrong".into(),
                    how_to_address: "Check connection settings and test endpoint".into(),
                    priority: Priority::Critical,
                });
            }
            
            // Check for complexity bias
            if self.detect_complexity_bias(reasoning) {
                problematic_patterns.push(ProblematicPattern {
                    pattern_id: "complexity_bias".into(),
                    description: "Preferring complex solution over simple alternatives".into(),
                    location: ReasoningLocation::Analysis,
                    correction: "Consider simpler solutions first".into(),
                    severity: Severity::Medium,
                });
            }
            
            let assessment = if problematic_patterns.iter().any(|p| p.severity == Severity::High) {
                ReasoningAssessment::Problematic { 
                    issues: problematic_patterns.iter().map(|p| p.description.clone()).collect()
                }
            } else {
                ReasoningAssessment::Acceptable { 
                    warnings: problematic_patterns.iter().map(|p| p.description.clone()).collect()
                }
            };
            
            ReasoningAnalysis {
                assessment,
                problematic_patterns,
                missing_considerations,
                perspective_suggestions: vec![
                    PerspectiveShift {
                        shift_type: ShiftType::QuestionAssumptions,
                        description: "Question whether installation is actually needed".into(),
                        new_perspective: "The real problem might be configuration, not missing software".into(),
                    },
                ],
            }
        }
        
        async fn generate_alternative_perspective(&self, context: &ProblemContext) -> AlternativePerspective {
            AlternativePerspective {
                alternative_problem_statement: "Connection configuration may be incorrect".into(),
                different_approach: "Verify and test configuration before installing anything".into(),
                evidence_to_gather: vec![
                    "Current connection configuration".into(),
                    "Expected vs actual endpoints".into(),
                    "Service status on expected endpoint".into(),
                ],
                simple_tests: vec![
                    SimpleTest {
                        description: "Check connection configuration".into(),
                        method: TestMethod::ConfigInspection,
                        expected_outcome: "Reveals correct endpoint to use".into(),
                    },
                    SimpleTest {
                        description: "Test connectivity to configured endpoint".into(),
                        method: TestMethod::ConnectivityTest,
                        expected_outcome: "Shows if service is reachable".into(),
                    },
                ],
            }
        }
    }
    
    /// Detects assumption-heavy reasoning
    pub struct AssumptionDetector;
    
    impl ReasoningPatternDetector for AssumptionDetector {
        fn id(&self) -> &str { "assumption_heavy" }
        
        async fn analyze_reasoning(&self, reasoning: &AgentReasoning) -> ReasoningAnalysis {
            let assumption_ratio = reasoning.assumptions.len() as f64 / reasoning.reasoning_chain.len() as f64;
            
            if assumption_ratio > 0.4 {  // More than 40% assumptions
                ReasoningAnalysis {
                    assessment: ReasoningAssessment::Problematic {
                        issues: vec!["Reasoning relies too heavily on unvalidated assumptions".into()],
                    },
                    problematic_patterns: vec![
                        ProblematicPattern {
                            pattern_id: "assumption_heavy".into(),
                            description: "High ratio of assumptions to validated facts".into(),
                            location: ReasoningLocation::Analysis,
                            correction: "Validate key assumptions before proceeding".into(),
                            severity: Severity::Medium,
                        }
                    ],
                    missing_considerations: reasoning.assumptions.iter().map(|assumption| {
                        MissingConsideration {
                            consideration: format!("Validation of: {}", assumption.content),
                            importance: "Unvalidated assumptions lead to wrong solutions".into(),
                            how_to_address: assumption.validation_method.clone(),
                            priority: Priority::High,
                        }
                    }).collect(),
                    perspective_suggestions: vec![
                        PerspectiveShift {
                            shift_type: ShiftType::ValidateAssumptions,
                            description: "Test assumptions before building on them".into(),
                            new_perspective: "Verify facts before inferring solutions".into(),
                        }
                    ],
                }
            } else {
                ReasoningAnalysis {
                    assessment: ReasoningAssessment::Acceptable { warnings: vec![] },
                    problematic_patterns: vec![],
                    missing_considerations: vec![],
                    perspective_suggestions: vec![],
                }
            }
        }
    }
    
    /// Detects momentum bias (sticking to initial approach)
    pub struct MomentumDetector;
    
    impl ReasoningPatternDetector for MomentumDetector {
        fn id(&self) -> &str { "momentum_bias" }
        
        async fn analyze_reasoning(&self, reasoning: &AgentReasoning) -> ReasoningAnalysis {
            // Look for signs of momentum bias:
            // - Limited consideration of alternatives
            // - Doubling down after failures  
            // - Dismissing simpler solutions
            
            let considers_alternatives = reasoning.reasoning_chain.iter()
                .any(|step| step.content.contains("alternative") || step.content.contains("instead"));
                
            let acknowledges_failure = reasoning.evidence.iter()
                .any(|evidence| evidence.indicates_failure());
                
            if acknowledges_failure && !considers_alternatives {
                ReasoningAnalysis {
                    assessment: ReasoningAssessment::Problematic {
                        issues: vec!["Not reconsidering approach after failure".into()],
                    },
                    problematic_patterns: vec![
                        ProblematicPattern {
                            pattern_id: "momentum_bias".into(),
                            description: "Continuing same approach despite failure signals".into(),
                            location: ReasoningLocation::ActionPlanning,
                            correction: "Step back and consider alternative approaches".into(),
                            severity: Severity::High,
                        }
                    ],
                    missing_considerations: vec![
                        MissingConsideration {
                            consideration: "Alternative problem interpretations".into(),
                            importance: "Current approach may be based on wrong problem understanding".into(),
                            how_to_address: "Restate problem from scratch without solution bias".into(),
                            priority: Priority::Critical,
                        }
                    ],
                    perspective_suggestions: vec![
                        PerspectiveShift {
                            shift_type: ShiftType::FreshStart,
                            description: "Start reasoning from scratch with failure evidence".into(),
                            new_perspective: "What if the original problem understanding was wrong?".into(),
                        }
                    ],
                }
            } else {
                ReasoningAnalysis {
                    assessment: ReasoningAssessment::Sound,
                    problematic_patterns: vec![],
                    missing_considerations: vec![],
                    perspective_suggestions: vec![],
                }
            }
        }
    }
}
```

### Integration with Existing Systems

```rust
/// Enhanced agent with reasoning supervision
pub struct ReasoningSupervisedAgent<A: Agent> {
    /// Base supervised agent
    base: SupervisedConscienceLayer<A>,
    
    /// Reasoning pattern supervisor
    reasoning_supervisor: ReasoningSupervisor,
}

impl<A: Agent> ReasoningSupervisedAgent<A> {
    /// Execute action with full supervision stack
    pub async fn execute_with_full_supervision(&mut self, task: &Task) -> Result<TaskResult> {
        // 1. Agent generates reasoning and proposed actions
        let reasoning = self.base.agent.generate_reasoning(task).await?;
        
        // 2. Reasoning supervisor validates the thinking process
        let reasoning_analysis = self.reasoning_supervisor.analyze_reasoning(&reasoning).await?;
        
        match reasoning_analysis.assessment {
            ReasoningAssessment::Sound => {
                // Proceed to tool call supervision
                self.execute_with_tool_supervision(&reasoning.proposed_actions).await
            }
            
            ReasoningAssessment::Acceptable { warnings } => {
                // Log warnings but proceed
                for warning in warnings {
                    self.log_reasoning_warning(&warning);
                }
                self.execute_with_tool_supervision(&reasoning.proposed_actions).await
            }
            
            ReasoningAssessment::Problematic { issues } => {
                // Generate alternative perspective and request user input
                let clean_context = self.reasoning_supervisor.clean_context
                    .generate_clean_context(&reasoning.problem_statement).await;
                    
                let alternative = self.reasoning_supervisor
                    .generate_alternative_perspective(&clean_context).await?;
                    
                // Present both perspectives to user
                self.present_reasoning_conflict(ReasoningConflict {
                    original_reasoning: reasoning,
                    issues_detected: issues,
                    alternative_perspective: alternative,
                    clean_context,
                }).await
            }
            
            ReasoningAssessment::Flawed { reason } => {
                // Block completely and restart with clean context
                Err(ReasoningError::FlawedReasoning { 
                    reason,
                    suggested_restart: true 
                })
            }
        }
    }
    
    /// Present reasoning conflict to user for resolution
    async fn present_reasoning_conflict(&self, conflict: ReasoningConflict) -> Result<TaskResult> {
        let message = format!(
            "I've detected potential issues with my reasoning approach:\n\n\
            **My Current Thinking:**\n{}\n\n\
            **Issues Detected:**\n{}\n\n\
            **Alternative Perspective:**\n{}\n\n\
            **Key Questions to Consider:**\n{}\n\n\
            How would you like me to proceed?",
            conflict.original_reasoning.problem_statement,
            conflict.issues_detected.join("\n- "),
            conflict.alternative_perspective.different_approach,
            conflict.clean_context.fundamental_questions.join("\n- ")
        );
        
        let user_decision = self.request_user_decision(&message, vec![
            Decision::UseOriginalApproach,
            Decision::UseAlternativeApproach,  
            Decision::CombineBothApproaches,
            Decision::StartOver,
        ]).await?;
        
        match user_decision {
            Decision::UseAlternativeApproach => {
                // Restart with alternative perspective
                self.restart_with_perspective(&conflict.alternative_perspective).await
            }
            Decision::UseOriginalApproach => {
                // Proceed with original but with warnings
                self.execute_with_tool_supervision(&conflict.original_reasoning.proposed_actions).await
            }
            // ... handle other decisions
        }
    }
}
```

## Example Reasoning Intervention

```
Agent Reasoning:
"Playwright connection failed to localhost:3000. I need to install browsers to fix this."

Supervisor Analysis:
- Pattern: install_escalation (HIGH severity)
- Missing: Configuration validation
- Alternative: "Connection target may be misconfigured"

Clean Context Questions:
- What endpoint should Playwright connect to?
- Is the expected service running?  
- Has this connection ever worked?

User Presentation:
"I detected a potential escalation trap in my reasoning. Instead of installing 
browsers, should I first verify the connection configuration and test if the 
expected service is actually running?"
```

## Relationships
- **Parent Nodes:** [elements/agent_conscience_pattern.md], [elements/supervisory_tool_validation.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/interruptible_agent_loops.md] - coordinates - With execution flow
  - [foundation/principles.md] - embodies - Thoughtful decision making

## Navigation Guidance  
- **Access Context:** Reference when implementing reasoning validation
- **Common Next Steps:** Review tool validation integration
- **Related Tasks:** Reasoning analysis, bias detection, perspective generation
- **Update Patterns:** Update when discovering new problematic reasoning patterns

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial reasoning pattern validation design with clean context generation