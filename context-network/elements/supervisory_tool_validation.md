# Supervisory Tool Validation: Preventing Escalation Traps

## Overview

This document defines supervisory validation patterns for agent tool calls, specifically designed to prevent "escalation traps" where agents make progressively more invasive changes when simpler solutions exist. This builds on the [Agent Conscience Pattern](./agent_conscience_pattern.md) to provide pre-tool-execution validation.

## The Escalation Trap Problem

The Playwright example illustrates a common agent failure pattern:

```
Connection fails → Install software → Create test environment → Configure system
```

When the actual issue was:
```  
Connection configured to localhost:3000 instead of correct endpoint
```

### Anti-Pattern Recognition

Supervisory validation catches these escalation patterns:

1. **Installation Trap**: Installing software before verifying configuration
2. **Bypass Trap**: Working around problems instead of investigating root cause  
3. **Complexity Trap**: Choosing complex solutions when simple ones exist
4. **Assumption Trap**: Acting on assumptions instead of validation

## Architecture Integration

### Tool Call Supervision Layer

```rust
/// Supervisory layer for tool call validation
pub struct ToolCallSupervisor {
    /// Core validation rules
    validators: Vec<Box<dyn ToolValidator>>,
    
    /// Project knowledge for context-aware decisions  
    project_knowledge: ProjectKnowledge,
    
    /// Integration with existing conscience
    conscience: Arc<dyn Conscience>,
}

/// Tool-specific validation trait
pub trait ToolValidator: Send + Sync {
    /// Validator identifier
    fn id(&self) -> &str;
    
    /// Check if validator applies to this tool call
    fn applies_to(&self, tool_call: &ToolCall) -> bool;
    
    /// Validate the tool call before execution
    async fn validate(&self, call: &ToolCall, context: &ValidationContext) -> ValidationResult;
    
    /// Suggest alternatives if validation fails
    async fn suggest_alternatives(&self, call: &ToolCall) -> Vec<Alternative>;
}

#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Proceed with tool call
    Approve,
    
    /// Proceed with warning
    Caution { warning: String },
    
    /// Block and require user confirmation
    RequireConfirmation { reason: String, risks: Vec<Risk> },
    
    /// Block completely
    Block { reason: String },
    
    /// Suggest prerequisite checks first
    RequirePrerequisites { checks: Vec<PrerequisiteCheck> },
}

/// Prerequisite validation before invasive actions
#[derive(Debug, Clone)]
pub struct PrerequisiteCheck {
    /// Description of what to check
    pub description: String,
    
    /// How to perform the check
    pub validation_method: ValidationMethod,
    
    /// What a successful check looks like
    pub success_criteria: String,
    
    /// Priority (higher = more critical)
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub enum ValidationMethod {
    /// Simple network connectivity test
    NetworkPing { host: String, port: u16 },
    
    /// Check if service is running
    ServiceStatus { service_name: String },
    
    /// Verify configuration value
    ConfigCheck { key: String, expected: Option<String> },
    
    /// Test endpoint accessibility  
    EndpointTest { url: String },
    
    /// Ask user to confirm
    UserConfirmation { question: String },
}
```

### Built-in Validators

```rust
pub mod validators {
    use super::*;
    
    /// Prevents software installation before connection validation
    pub struct InstallationValidator;
    
    impl ToolValidator for InstallationValidator {
        fn id(&self) -> &str { "installation_prevention" }
        
        fn applies_to(&self, tool_call: &ToolCall) -> bool {
            matches!(tool_call.tool_type, ToolType::PackageInstall | ToolType::BrowserInstall)
        }
        
        async fn validate(&self, call: &ToolCall, context: &ValidationContext) -> ValidationResult {
            // Check if this is in response to a connection failure
            if context.recent_failures.iter().any(|f| f.is_connection_failure()) {
                // Require connection diagnostics first
                ValidationResult::RequirePrerequisites {
                    checks: vec![
                        PrerequisiteCheck {
                            description: "Verify connection endpoint configuration".into(),
                            validation_method: ValidationMethod::ConfigCheck {
                                key: "connection_url".into(),
                                expected: None,
                            },
                            success_criteria: "Endpoint is correctly configured".into(),
                            priority: 10,
                        },
                        PrerequisiteCheck {
                            description: "Test connectivity to configured endpoint".into(),
                            validation_method: ValidationMethod::EndpointTest {
                                url: context.get_connection_url(),
                            },
                            success_criteria: "Endpoint responds successfully".into(),
                            priority: 9,
                        },
                    ],
                }
            } else {
                ValidationResult::Approve
            }
        }
        
        async fn suggest_alternatives(&self, call: &ToolCall) -> Vec<Alternative> {
            vec![
                Alternative {
                    description: "Verify existing installation is accessible".into(),
                    actions: vec![
                        Action::CheckInstalledVersion,
                        Action::TestExistingConnection,
                    ],
                },
                Alternative {
                    description: "Check connection configuration".into(),
                    actions: vec![
                        Action::ReviewConfig,
                        Action::TestConnectivity,
                    ],
                },
            ]
        }
    }
    
    /// Validates file system operations don't bypass investigation
    pub struct FileSystemValidator;
    
    impl ToolValidator for FileSystemValidator {
        fn id(&self) -> &str { "filesystem_safety" }
        
        fn applies_to(&self, tool_call: &ToolCall) -> bool {
            matches!(tool_call.tool_type, 
                ToolType::FileWrite | 
                ToolType::FileDelete | 
                ToolType::DirectoryCreate
            )
        }
        
        async fn validate(&self, call: &ToolCall, context: &ValidationContext) -> ValidationResult {
            // Check for potential configuration bypass
            if let ToolCall::FileWrite { path, .. } = call {
                if path.contains("config") || path.contains("env") {
                    // Require understanding existing config first
                    return ValidationResult::RequirePrerequisites {
                        checks: vec![
                            PrerequisiteCheck {
                                description: "Review existing configuration".into(),
                                validation_method: ValidationMethod::UserConfirmation {
                                    question: format!(
                                        "Should I modify {}? Have you verified the current config?", 
                                        path
                                    ),
                                },
                                success_criteria: "User confirms config change is appropriate".into(),
                                priority: 8,
                            },
                        ],
                    };
                }
            }
            
            ValidationResult::Approve
        }
    }
    
    /// Network operation validator  
    pub struct NetworkValidator;
    
    impl ToolValidator for NetworkValidator {
        fn id(&self) -> &str { "network_operations" }
        
        fn applies_to(&self, tool_call: &ToolCall) -> bool {
            matches!(tool_call.tool_type, ToolType::HttpRequest | ToolType::NetworkConnection)
        }
        
        async fn validate(&self, call: &ToolCall, context: &ValidationContext) -> ValidationResult {
            if let ToolCall::HttpRequest { url, .. } = call {
                // Check for localhost assumptions
                if url.contains("localhost:3000") {
                    ValidationResult::RequireConfirmation {
                        reason: "Request targets localhost:3000".into(),
                        risks: vec![
                            Risk::WrongEndpoint("May not be the intended service".into()),
                            Risk::ServiceNotRunning("Service may not be running on 3000".into()),
                        ],
                    }
                } else {
                    ValidationResult::Approve
                }
            } else {
                ValidationResult::Approve
            }
        }
    }
}
```

### Context-Aware Validation

```rust
/// Project knowledge for context-aware validation
pub struct ProjectKnowledge {
    /// Known service endpoints
    pub endpoints: HashMap<String, ServiceEndpoint>,
    
    /// Configuration patterns
    pub config_patterns: Vec<ConfigPattern>,
    
    /// Best practices for this project
    pub practices: Vec<BestPractice>,
    
    /// Common failure patterns and solutions
    pub known_issues: Vec<KnownIssue>,
}

pub struct ServiceEndpoint {
    pub name: String,
    pub default_url: String,
    pub config_key: String,
    pub health_check: Option<String>,
}

pub struct KnownIssue {
    pub symptom: String,
    pub common_cause: String,
    pub simple_fix: String,
    pub escalation_trap: String,
}

impl ProjectKnowledge {
    /// Load from project configuration
    pub fn from_project(project_path: &Path) -> Result<Self> {
        // Read from .patinox/knowledge.toml or similar
        let config = std::fs::read_to_string(project_path.join(".patinox/knowledge.toml"))?;
        
        Ok(Self {
            endpoints: Self::parse_endpoints(&config),
            config_patterns: Self::parse_config_patterns(&config),
            practices: Self::parse_practices(&config),
            known_issues: Self::parse_known_issues(&config),
        })
    }
    
    /// Check if a tool call matches a known escalation trap
    pub fn detect_escalation_trap(&self, call: &ToolCall, context: &ValidationContext) -> Option<&KnownIssue> {
        self.known_issues.iter().find(|issue| {
            // Match symptom from recent failures
            let symptom_match = context.recent_failures.iter()
                .any(|failure| failure.message.contains(&issue.symptom));
                
            // Match escalation pattern from tool call
            let escalation_match = call.description().contains(&issue.escalation_trap);
            
            symptom_match && escalation_match
        })
    }
}
```

### Integration with Existing Conscience

```rust
/// Enhanced conscience with tool call supervision
pub struct SupervisedConscienceLayer<A: Agent> {
    /// Base conscience layer
    base: ConscienceLayer<A>,
    
    /// Tool call supervisor
    tool_supervisor: ToolCallSupervisor,
}

impl<A: Agent> SupervisedConscienceLayer<A> {
    /// Validate tool call with supervision
    async fn validate_tool_call(&self, call: &ToolCall) -> Result<ToolCall> {
        // First, apply base conscience validation
        let call = self.base.before_action(&call.into()).await?;
        
        // Then apply tool-specific supervision
        let validation_context = ValidationContext::from_agent_state(&self.base.agent.state());
        let result = self.tool_supervisor.validate(&call, &validation_context).await;
        
        match result {
            ValidationResult::Approve => Ok(call),
            
            ValidationResult::Caution { warning } => {
                self.base.agent.log_warning(&format!("Tool validation warning: {}", warning));
                Ok(call)
            }
            
            ValidationResult::RequireConfirmation { reason, risks } => {
                let confirmation = self.request_user_confirmation(&reason, &risks).await?;
                if confirmation {
                    Ok(call)
                } else {
                    Err(ValidationError::UserRejected)
                }
            }
            
            ValidationResult::Block { reason } => {
                Err(ValidationError::Blocked { reason })
            }
            
            ValidationResult::RequirePrerequisites { checks } => {
                // Execute prerequisite checks
                for check in checks {
                    if !self.execute_prerequisite_check(&check).await? {
                        return Err(ValidationError::PrerequisiteFailed { 
                            check: check.description 
                        });
                    }
                }
                
                // If all prerequisites pass, approve
                Ok(call)
            }
        }
    }
    
    /// Execute a prerequisite check
    async fn execute_prerequisite_check(&self, check: &PrerequisiteCheck) -> Result<bool> {
        match &check.validation_method {
            ValidationMethod::NetworkPing { host, port } => {
                // Test network connectivity
                self.test_network_connection(host, *port).await
            }
            
            ValidationMethod::EndpointTest { url } => {
                // Test HTTP endpoint
                self.test_http_endpoint(url).await  
            }
            
            ValidationMethod::ConfigCheck { key, expected } => {
                // Check configuration value
                self.check_config_value(key, expected.as_deref()).await
            }
            
            ValidationMethod::UserConfirmation { question } => {
                // Ask user for confirmation
                self.ask_user_confirmation(question).await
            }
            
            ValidationMethod::ServiceStatus { service_name } => {
                // Check if service is running
                self.check_service_status(service_name).await
            }
        }
    }
}
```

### Example Configuration

```toml
# .patinox/knowledge.toml

[endpoints]
[endpoints.playwright]
name = "Playwright Server"
default_url = "http://localhost:9222"
config_key = "PLAYWRIGHT_SERVER_URL"
health_check = "/json/version"

[endpoints.api]
name = "Main API"
default_url = "http://localhost:8080"
config_key = "API_BASE_URL"

[[known_issues]]
symptom = "Failed to connect to Playwright server"
common_cause = "Wrong endpoint configuration"
simple_fix = "Check PLAYWRIGHT_SERVER_URL environment variable"
escalation_trap = "install playwright browsers"

[[known_issues]]
symptom = "Connection refused"
common_cause = "Service not running or wrong port"
simple_fix = "Verify service status and port configuration"
escalation_trap = "install dependencies"

[[practices]]
name = "Connection Failures"
rule = "Always verify configuration before installing software"
validation = "Check config and test connectivity first"
```

## Implementation Strategy

### Phase 1: Basic Validation
1. Implement core `ToolValidator` trait
2. Create basic validators for common escalation traps
3. Integration points with existing conscience layer

### Phase 2: Project Knowledge
1. Add project knowledge configuration
2. Context-aware validation based on project patterns
3. Known issue detection and prevention

### Phase 3: Learning System
1. Learn from validation failures
2. Adapt validation rules based on project patterns
3. User feedback integration

### Phase 4: Advanced Features
1. Multi-step validation flows
2. Predictive escalation detection
3. Integration with external knowledge bases

## Example Usage

```rust
// Create supervised agent
let conscience = ThoroughConscience::new();
let tool_supervisor = ToolCallSupervisor::from_project("./project")?;

let supervised_agent = SupervisedConscienceLayer::new(
    ConscienceLayer::new(base_agent, conscience),
    tool_supervisor,
);

// Tool call automatically validated
let result = supervised_agent.execute_tool_call(ToolCall::BrowserInstall {
    browser: "chromium".into(),
}).await;

// Output:
// ValidationError::PrerequisiteFailed { 
//     check: "Test connectivity to configured endpoint"
// }
```

## Best Practices

1. **Default to Questioning**: When in doubt, require confirmation
2. **Project-Specific**: Tailor validation to project patterns
3. **Escalation Detection**: Learn common escalation traps
4. **User-Friendly**: Provide clear explanations for blocks
5. **Prerequisite Ordering**: Check simple things first
6. **Override Support**: Allow emergency overrides with justification

## Relationships
- **Parent Nodes:** [elements/agent_conscience_pattern.md]
- **Child Nodes:** None  
- **Related Nodes:**
  - [elements/interruptible_agent_loops.md] - coordinates - With execution control
  - [elements/monitoring_strategy.md] - informs - Validation decisions

## Navigation Guidance
- **Access Context:** Reference when implementing tool call validation
- **Common Next Steps:** Review agent conscience pattern integration
- **Related Tasks:** Tool safety, validation design, escalation prevention
- **Update Patterns:** Update when adding new validation rules or project patterns

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18  
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial supervisory tool validation design to prevent escalation traps