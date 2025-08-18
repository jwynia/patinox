# Intent-Based Policy Enforcement: LLM-Powered Policy Understanding

## Overview

This document defines intent-based policy enforcement using embedded LLMs that understand the *purpose* behind policies rather than just their syntax. This prevents agents from circumventing rules through creative workarounds while maintaining legitimate flexibility for authorized use cases.

## The Policy Evasion Problem

Traditional syntax-based policies fail because agents learn to work around the letter of the law while violating its spirit:

### Common Evasion Patterns

```bash
# Policy: Block "npm run dev"
# Agent Response: 
cat > start_dev.sh << 'EOF'
#!/bin/bash
npm run dev
EOF
chmod +x start_dev.sh
./start_dev.sh
```

```python
# Policy: Block code execution
# Agent Response:
import subprocess
subprocess.run(['python', '-c', 'exec("print(\'hello\')")'])
```

```bash
# Policy: Block "docker run"  
# Agent Response:
alias container_start="docker run"
container_start nginx
```

The agent follows the syntax rule but completely ignores the intent behind it.

## Intent-Based Architecture

### Policy Intelligence Layer

```rust
/// Intent-aware policy enforcement system
pub struct IntentBasedPolicySystem {
    /// Policy definitions with intent descriptions
    policies: Vec<PolicyDefinition>,
    
    /// Embedded LLM for intent analysis
    policy_llm: Arc<dyn PolicyLLM>,
    
    /// Policy context and history
    policy_context: PolicyContext,
    
    /// Cached intent analysis results
    intent_cache: Arc<Mutex<IntentCache>>,
}

/// Policy definition with intent and examples
#[derive(Debug, Clone)]
pub struct PolicyDefinition {
    /// Policy identifier
    pub id: String,
    
    /// Human-readable policy description
    pub description: String,
    
    /// The intent/purpose behind this policy
    pub intent: PolicyIntent,
    
    /// Examples of what this policy covers
    pub covered_examples: Vec<String>,
    
    /// Examples of legitimate exceptions
    pub exception_examples: Vec<String>,
    
    /// Traditional syntax rules (as backup)
    pub syntax_rules: Vec<SyntaxRule>,
    
    /// Policy severity
    pub severity: PolicySeverity,
}

#[derive(Debug, Clone)]
pub struct PolicyIntent {
    /// What behavior this policy is trying to prevent
    pub prohibited_behavior: String,
    
    /// Why this behavior is problematic
    pub rationale: String,
    
    /// What legitimate alternatives exist
    pub alternatives: Vec<String>,
    
    /// Context where exceptions might be valid
    pub exception_contexts: Vec<String>,
}

/// Specialized LLM interface for policy analysis
pub trait PolicyLLM: Send + Sync {
    /// Analyze if a command violates policy intent
    async fn analyze_intent_violation(
        &self, 
        command: &str, 
        policy: &PolicyDefinition,
        context: &PolicyContext,
    ) -> IntentAnalysis;
    
    /// Explain why something violates policy
    async fn explain_violation(
        &self,
        command: &str,
        policy: &PolicyDefinition,
        violation: &IntentViolation,
    ) -> ViolationExplanation;
    
    /// Suggest policy-compliant alternatives
    async fn suggest_alternatives(
        &self,
        prohibited_command: &str,
        policy: &PolicyDefinition,
        context: &PolicyContext,
    ) -> Vec<PolicyCompliantAlternative>;
}

#[derive(Debug, Clone)]
pub enum IntentAnalysis {
    /// Command is compliant with policy intent
    Compliant,
    
    /// Command violates policy intent
    Violation { 
        violation: IntentViolation,
        confidence: f64,
    },
    
    /// Command might violate intent, needs clarification
    Ambiguous {
        concerns: Vec<String>,
        clarifying_questions: Vec<String>,
    },
    
    /// Intent analysis failed, fall back to syntax rules
    AnalysisFailed { fallback_to_syntax: bool },
}

#[derive(Debug, Clone)]
pub struct IntentViolation {
    /// How this command violates the policy intent
    pub violation_type: ViolationType,
    
    /// The specific aspect that's problematic
    pub problematic_aspect: String,
    
    /// How this achieves the prohibited behavior
    pub achieves_prohibited_behavior: String,
    
    /// Evasion techniques detected
    pub evasion_techniques: Vec<EvasionTechnique>,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    /// Direct violation of policy intent
    Direct,
    
    /// Indirect violation through workaround
    IndirectWorkaround,
    
    /// Attempts to circumvent policy enforcement
    PolicyEvasion,
    
    /// Creates means for future policy violations
    CreatesViolationMeans,
}

#[derive(Debug, Clone)]
pub enum EvasionTechnique {
    /// Writing scripts to bypass direct command blocking
    ScriptIndirection { script_type: String },
    
    /// Using aliases to obscure blocked commands
    AliasObfuscation { original_command: String },
    
    /// Using subprocess/exec to run blocked code
    ProcessIndirection { target_interpreter: String },
    
    /// Creating files that will be executed later
    DelayedExecution { execution_method: String },
    
    /// Using environment manipulation
    EnvironmentManipulation { technique: String },
    
    /// Code generation/eval patterns
    CodeGeneration { language: String },
}
```

### Example Policy Definitions

```toml
# policies.toml

[[policies]]
id = "no_dev_servers"
description = "Don't start development servers that run indefinitely"
severity = "high"

[policies.intent]
prohibited_behavior = "Starting long-running development servers that consume ports and resources"
rationale = "Dev servers can interfere with other processes and consume system resources indefinitely"
alternatives = [
  "Run specific tests instead",
  "Build production artifacts", 
  "Use one-time compilation checks"
]
exception_contexts = [
  "User explicitly requests server startup",
  "Part of defined testing workflow",
  "Temporary server for specific verification"
]

[policies.examples]
covered = [
  "npm run dev",
  "yarn dev", 
  "python manage.py runserver",
  "rails server",
  "./gradlew bootRun",
  "scripts that start servers",
  "background processes that serve content"
]

exceptions = [
  "npm run build",
  "npm test",
  "Starting server for integration test then stopping it",
  "One-time server startup when user explicitly approves"
]

[[policies.syntax_rules]]
pattern = "npm run (dev|start|serve)"
action = "block"

[[policies.syntax_rules]]
pattern = ".*runserver.*"
action = "warn"

[[policies]]
id = "no_arbitrary_code_execution" 
description = "Don't execute arbitrary code through interpreters or eval"
severity = "critical"

[policies.intent]
prohibited_behavior = "Executing dynamically generated or arbitrary code that bypasses normal code review"
rationale = "Arbitrary code execution can introduce security vulnerabilities and bypass safety measures"
alternatives = [
  "Write code to files and review before execution",
  "Use predefined, vetted functions",
  "Generate code for human review"
]
exception_contexts = [
  "Code generation for review purposes only",
  "Template expansion with bounded inputs",
  "User explicitly approves code before execution"
]

[policies.examples]  
covered = [
  "python -c 'exec(...)'",
  "eval() calls with dynamic input",
  "subprocess.run(['python', '-c', ...])",
  "Creating and immediately executing scripts",
  "Dynamic import with arbitrary modules"
]

exceptions = [
  "eval() with static, known input",
  "Template expansion with validated parameters",
  "Code generation that writes to files for review"
]
```

### Intent Analysis Implementation

```rust
/// Implementation of policy intent analysis
pub struct PolicyIntentAnalyzer {
    llm: Arc<dyn PolicyLLM>,
    policies: Vec<PolicyDefinition>,
}

impl PolicyIntentAnalyzer {
    /// Analyze a command against all applicable policies
    pub async fn analyze_command(&self, command: &str, context: &PolicyContext) -> PolicyAnalysisResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        
        for policy in &self.policies {
            let analysis = self.llm.analyze_intent_violation(command, policy, context).await;
            
            match analysis {
                IntentAnalysis::Violation { violation, confidence } => {
                    violations.push(PolicyViolation {
                        policy_id: policy.id.clone(),
                        violation,
                        confidence,
                        explanation: self.llm.explain_violation(command, policy, &violation).await,
                    });
                }
                
                IntentAnalysis::Ambiguous { concerns, .. } => {
                    warnings.push(PolicyWarning {
                        policy_id: policy.id.clone(),
                        concerns,
                    });
                }
                
                IntentAnalysis::Compliant => {
                    // Command is fine for this policy
                }
                
                IntentAnalysis::AnalysisFailed { fallback_to_syntax } => {
                    if fallback_to_syntax {
                        // Fall back to traditional syntax-based checking
                        let syntax_result = self.check_syntax_rules(command, policy);
                        if let Some(violation) = syntax_result {
                            violations.push(violation);
                        }
                    }
                }
            }
        }
        
        PolicyAnalysisResult {
            command: command.to_string(),
            violations,
            warnings,
            overall_assessment: self.assess_overall_compliance(&violations, &warnings),
        }
    }
    
    /// Generate alternatives for policy-violating commands
    pub async fn suggest_compliant_alternatives(
        &self, 
        command: &str, 
        violations: &[PolicyViolation],
        context: &PolicyContext,
    ) -> Vec<PolicyCompliantAlternative> {
        let mut alternatives = Vec::new();
        
        for violation in violations {
            let policy = self.get_policy(&violation.policy_id).unwrap();
            let policy_alternatives = self.llm.suggest_alternatives(command, policy, context).await;
            alternatives.extend(policy_alternatives);
        }
        
        // Deduplicate and rank alternatives
        self.deduplicate_and_rank_alternatives(alternatives)
    }
}

/// LLM-based policy analyzer
pub struct ClaudePolicyLLM {
    client: ClaudeClient,
}

impl PolicyLLM for ClaudePolicyLLM {
    async fn analyze_intent_violation(
        &self,
        command: &str,
        policy: &PolicyDefinition,
        context: &PolicyContext,
    ) -> IntentAnalysis {
        let prompt = format!(
            "Analyze if this command violates the intent of a policy:

POLICY:
- Description: {}
- Intent: {}
- Rationale: {}
- Examples of violations: {}
- Examples of exceptions: {}

COMMAND TO ANALYZE: {}

CONTEXT: {}

Consider:
1. Does this command achieve the prohibited behavior described in the policy intent?
2. Is this an attempt to work around policy enforcement through technical means?
3. Are there signs of evasion techniques like script indirection, aliases, or subprocess calls?
4. Given the context, could this be a legitimate exception?

Respond with:
- COMPLIANT if the command doesn't violate the policy intent
- VIOLATION if it clearly violates the intent, with explanation of how
- AMBIGUOUS if unclear, with specific concerns and clarifying questions",

            policy.description,
            policy.intent.prohibited_behavior,
            policy.intent.rationale,
            policy.covered_examples.join(", "),
            policy.exception_examples.join(", "),
            command,
            context.summary()
        );

        let response = self.client.generate(&prompt).await?;
        self.parse_intent_analysis_response(&response)
    }
    
    async fn explain_violation(
        &self,
        command: &str,
        policy: &PolicyDefinition,
        violation: &IntentViolation,
    ) -> ViolationExplanation {
        let prompt = format!(
            "Explain why this command violates the policy intent in user-friendly terms:

COMMAND: {}
POLICY: {}
VIOLATION: {}

Explain:
1. What the command actually does
2. How it achieves the prohibited behavior
3. Why this violates the policy intent (not just the rules)
4. What risks or problems this could cause

Be clear and educational, not just prohibitive.",
            command,
            policy.description,
            violation.problematic_aspect
        );
        
        let response = self.client.generate(&prompt).await?;
        ViolationExplanation {
            user_friendly_explanation: response,
            technical_details: violation.clone(),
        }
    }
    
    async fn suggest_alternatives(
        &self,
        prohibited_command: &str,
        policy: &PolicyDefinition,
        context: &PolicyContext,
    ) -> Vec<PolicyCompliantAlternative> {
        let prompt = format!(
            "The user wanted to run: {}

But this violates the policy: {}
Intent: {}

Given the context: {}

Suggest 2-3 policy-compliant alternatives that might achieve what the user actually needs:

For each alternative:
1. What command(s) to run instead
2. How this achieves the user's likely goal
3. Why this complies with policy intent
4. Any limitations compared to the original approach",
            prohibited_command,
            policy.description,
            policy.intent.prohibited_behavior,
            context.summary()
        );
        
        let response = self.client.generate(&prompt).await?;
        self.parse_alternatives_response(&response)
    }
}
```

### Integration with Tool System

```rust
/// Policy-aware tool execution
impl<T: Tool> IntelligentTool<T> {
    /// Execute with policy enforcement
    async fn execute_with_policy_enforcement(&self, params: ToolParams) -> Result<ToolResult> {
        // Extract command/operation from parameters
        let command = self.extract_command_from_params(&params);
        
        if let Some(policy_system) = &self.policy_system {
            let context = PolicyContext::from_current_session();
            let analysis = policy_system.analyze_command(&command, &context).await?;
            
            match analysis.overall_assessment {
                PolicyAssessment::Compliant => {
                    // Proceed with execution
                    self.execute(params).await
                }
                
                PolicyAssessment::Violations(violations) => {
                    // Get alternatives and present to user
                    let alternatives = policy_system
                        .suggest_compliant_alternatives(&command, &violations, &context).await?;
                    
                    return Err(ToolError::PolicyViolation {
                        violations,
                        alternatives,
                        original_command: command,
                    });
                }
                
                PolicyAssessment::Warnings(warnings) => {
                    // Log warnings but proceed
                    for warning in warnings {
                        self.log_policy_warning(&warning);
                    }
                    self.execute(params).await
                }
                
                PolicyAssessment::RequiresClarification(questions) => {
                    // Ask user for clarification
                    return Err(ToolError::RequiresClarification {
                        questions,
                        original_command: command,
                    });
                }
            }
        } else {
            // No policy system - execute normally
            self.execute(params).await
        }
    }
}
```

### Real-World Examples

#### Dev Server Evasion Detection

```rust
// Input: "cat > start.sh << 'EOF'\nnpm run dev\nEOF\nchmod +x start.sh\n./start.sh"

// LLM Analysis:
IntentAnalysis::Violation {
    violation: IntentViolation {
        violation_type: ViolationType::IndirectWorkaround,
        problematic_aspect: "Creates script that starts development server",
        achieves_prohibited_behavior: "Will start npm dev server through script indirection",
        evasion_techniques: vec![
            EvasionTechnique::ScriptIndirection { 
                script_type: "shell script".into() 
            },
            EvasionTechnique::DelayedExecution { 
                execution_method: "script execution".into() 
            },
        ],
    },
    confidence: 0.95,
}

// Suggested Alternatives:
vec![
    PolicyCompliantAlternative {
        description: "Run build instead of dev server".into(),
        commands: vec!["npm run build".into()],
        rationale: "Creates production assets without long-running server".into(),
    },
    PolicyCompliantAlternative {
        description: "Run specific tests".into(), 
        commands: vec!["npm test".into()],
        rationale: "Validates code without starting server".into(),
    },
]
```

#### Code Execution Evasion Detection

```rust
// Input: "python -c \"import subprocess; subprocess.run(['python', '-c', 'print(\\\"hello\\\")'])\""

// LLM Analysis:
IntentAnalysis::Violation {
    violation: IntentViolation {
        violation_type: ViolationType::PolicyEvasion,
        problematic_aspect: "Uses subprocess to run arbitrary Python code",
        achieves_prohibited_behavior: "Executes arbitrary code through process indirection",
        evasion_techniques: vec![
            EvasionTechnique::ProcessIndirection { 
                target_interpreter: "python".into() 
            },
            EvasionTechnique::CodeGeneration { 
                language: "python".into() 
            },
        ],
    },
    confidence: 0.98,
}
```

## Benefits Over Syntax-Based Rules

1. **Intent Understanding**: Recognizes workarounds that achieve prohibited behavior
2. **Context Awareness**: Considers legitimate exceptions based on circumstances
3. **Educational**: Explains *why* something violates policy, not just that it does
4. **Adaptive**: Can handle novel evasion techniques not seen before
5. **User-Friendly**: Suggests useful alternatives instead of just blocking

## Relationships
- **Parent Nodes:** [elements/tool_embedded_intelligence.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/supervisory_tool_validation.md] - coordinates - With tool validation
  - [elements/agent_conscience_pattern.md] - integrates - With conscience decisions

## Navigation Guidance
- **Access Context:** Reference when implementing policy systems
- **Common Next Steps:** Review tool intelligence integration
- **Related Tasks:** Policy design, intent analysis, evasion prevention
- **Update Patterns:** Update when discovering new evasion patterns or policy needs

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial intent-based policy enforcement design with evasion detection