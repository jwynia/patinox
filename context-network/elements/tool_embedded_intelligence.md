# Tool-Embedded Intelligence: Optional LLM-Enhanced Tools

## Overview

This document defines architecture for embedding lightweight LLM capabilities directly into tools, providing optional contextual validation, self-awareness, and intelligent behavior. The core principle: **freedom to use enhanced capabilities without obligation** - tools work perfectly without LLM enhancement, but become more intelligent when enhanced.

## The Freedom-Without-Obligation Principle

### Core Tenets

1. **Base Functionality Independent**: Every tool must work perfectly without LLM enhancement
2. **Optional Enhancement**: LLM capabilities are additive, not fundamental
3. **Graceful Degradation**: Tools detect LLM availability and adapt behavior accordingly
4. **User Choice**: Users control which tools have LLM capabilities enabled
5. **Performance Conscious**: LLM enhancement doesn't impact base tool performance

### Anti-Patterns to Avoid

- ❌ Tools that require LLM to function at all
- ❌ "Smart" tools that are actually dumber without LLM
- ❌ Always-on LLM usage that impacts performance
- ❌ Mandatory enhancement that removes user choice
- ❌ Black box intelligence that obscures tool behavior

## Architecture

### Tool Intelligence Layer

```rust
/// Optional intelligence layer for tools
pub trait ToolIntelligence: Send + Sync {
    /// Check if this tool call makes sense in context
    async fn validate_call(&self, call: &ToolCall, context: &ToolContext) -> ValidationAdvice;
    
    /// Suggest better alternatives if available
    async fn suggest_alternatives(&self, call: &ToolCall) -> Vec<AlternativeAction>;
    
    /// Explain what this tool call will actually do
    async fn explain_impact(&self, call: &ToolCall) -> ImpactExplanation;
    
    /// Detect potential issues before execution
    async fn detect_issues(&self, call: &ToolCall, context: &ToolContext) -> Vec<PotentialIssue>;
}

/// Base tool trait - works without intelligence
pub trait Tool: Send + Sync {
    /// Tool identifier
    fn id(&self) -> &str;
    
    /// Execute the tool (core functionality)
    async fn execute(&self, params: ToolParams) -> Result<ToolResult>;
    
    /// Tool capabilities description
    fn capabilities(&self) -> ToolCapabilities;
    
    /// Optional: Attach intelligence layer
    fn with_intelligence(self, intelligence: Box<dyn ToolIntelligence>) -> IntelligentTool<Self>
    where
        Self: Sized,
    {
        IntelligentTool {
            base_tool: self,
            intelligence: Some(intelligence),
        }
    }
    
    /// Check if intelligence is available
    fn has_intelligence(&self) -> bool {
        false  // Base tools don't have intelligence
    }
}

/// Tool with optional intelligence enhancement
pub struct IntelligentTool<T: Tool> {
    base_tool: T,
    intelligence: Option<Box<dyn ToolIntelligence>>,
}

impl<T: Tool> IntelligentTool<T> {
    /// Create tool with intelligence disabled
    pub fn without_intelligence(base_tool: T) -> Self {
        Self {
            base_tool,
            intelligence: None,
        }
    }
    
    /// Enable intelligence if available
    pub async fn maybe_enable_intelligence(&mut self, llm_provider: &dyn LLMProvider) -> bool {
        if llm_provider.is_available() {
            let intelligence = self.create_intelligence(llm_provider).await;
            self.intelligence = Some(intelligence);
            true
        } else {
            false
        }
    }
}

impl<T: Tool> Tool for IntelligentTool<T> {
    fn id(&self) -> &str {
        self.base_tool.id()
    }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        // Intelligence is optional - base functionality always works
        if let Some(intelligence) = &self.intelligence {
            // Optional pre-execution validation
            if let Some(validation) = self.maybe_validate(&params).await {
                if validation.should_block() {
                    return Err(ToolError::ValidationFailed(validation.reason));
                }
                if validation.has_warnings() {
                    // Log warnings but continue
                    self.log_warnings(&validation.warnings);
                }
            }
        }
        
        // Core execution (always works)
        self.base_tool.execute(params).await
    }
    
    fn has_intelligence(&self) -> bool {
        self.intelligence.is_some()
    }
}
```

### Specific Tool Intelligence Examples

#### File System Tool Intelligence

```rust
/// File system tool with optional intelligence
pub struct FileSystemTool {
    base_fs: FileSystem,
}

impl Tool for FileSystemTool {
    fn id(&self) -> &str { "filesystem" }
    
    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        match params {
            ToolParams::WriteFile { path, content } => {
                self.base_fs.write_file(&path, &content).await
            }
            ToolParams::ReadFile { path } => {
                self.base_fs.read_file(&path).await
            }
            ToolParams::DeleteFile { path } => {
                self.base_fs.delete_file(&path).await
            }
        }
    }
}

/// Intelligence layer for file system operations
pub struct FileSystemIntelligence {
    llm: Arc<dyn LLMProvider>,
}

impl ToolIntelligence for FileSystemIntelligence {
    async fn validate_call(&self, call: &ToolCall, context: &ToolContext) -> ValidationAdvice {
        if let ToolCall::WriteFile { path, content } = call {
            // Check if this might overwrite something important
            if path.contains("config") || path.ends_with(".env") {
                let prompt = format!(
                    "I'm about to write to {}. Given the context that {}, does this seem appropriate? \
                    The content starts with: {}...",
                    path,
                    context.current_task,
                    content.chars().take(100).collect::<String>()
                );
                
                let response = self.llm.generate(&prompt).await?;
                
                if response.contains("inappropriate") || response.contains("dangerous") {
                    return ValidationAdvice::Block {
                        reason: "LLM detected potential config overwrite risk".into(),
                        suggestion: "Consider backing up existing file first".into(),
                    };
                }
            }
        }
        
        ValidationAdvice::Proceed
    }
    
    async fn suggest_alternatives(&self, call: &ToolCall) -> Vec<AlternativeAction> {
        if let ToolCall::WriteFile { path, .. } = call {
            if self.file_exists(path).await {
                vec![
                    AlternativeAction {
                        description: "Back up existing file first".into(),
                        actions: vec![
                            Action::ReadFile { path: path.clone() },
                            Action::WriteFile { 
                                path: format!("{}.backup", path), 
                                content: "".into() // Would be filled from read
                            },
                        ],
                    },
                    AlternativeAction {
                        description: "Append instead of overwriting".into(),
                        actions: vec![
                            Action::AppendFile { path: path.clone(), content: "".into() },
                        ],
                    },
                ]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
    
    async fn explain_impact(&self, call: &ToolCall) -> ImpactExplanation {
        match call {
            ToolCall::WriteFile { path, content } => {
                let file_exists = self.file_exists(path).await;
                let size_estimate = content.len();
                
                ImpactExplanation {
                    summary: if file_exists {
                        format!("Will overwrite existing file {} ({} bytes)", path, size_estimate)
                    } else {
                        format!("Will create new file {} ({} bytes)", path, size_estimate)
                    },
                    details: vec![
                        format!("File location: {}", path),
                        format!("Content size: {} bytes", size_estimate),
                        if file_exists { "⚠️ Existing content will be lost".into() } else { "✅ New file creation".into() },
                    ],
                    risk_level: if file_exists { RiskLevel::Medium } else { RiskLevel::Low },
                }
            }
            
            ToolCall::DeleteFile { path } => {
                ImpactExplanation {
                    summary: format!("Will permanently delete file {}", path),
                    details: vec![
                        "⚠️ This action cannot be undone".into(),
                        "Consider backing up the file first".into(),
                    ],
                    risk_level: RiskLevel::High,
                }
            }
            
            _ => ImpactExplanation::minimal("Standard file operation"),
        }
    }
}
```

#### Network Tool Intelligence

```rust
/// Network tool with connection intelligence
pub struct NetworkTool {
    client: HttpClient,
}

impl Tool for NetworkTool {
    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        match params {
            ToolParams::HttpRequest { url, method, headers, body } => {
                self.client.request(method, &url, headers, body).await
            }
            ToolParams::Ping { host } => {
                self.client.ping(&host).await
            }
        }
    }
}

/// Intelligence for network operations
pub struct NetworkIntelligence {
    llm: Arc<dyn LLMProvider>,
    known_endpoints: HashMap<String, EndpointInfo>,
}

impl ToolIntelligence for NetworkIntelligence {
    async fn validate_call(&self, call: &ToolCall, context: &ToolContext) -> ValidationAdvice {
        if let ToolCall::HttpRequest { url, .. } = call {
            // Check for common endpoint mistakes
            if url.contains("localhost:3000") {
                let prompt = format!(
                    "The request is targeting localhost:3000, but the context suggests {}. \
                    Known endpoints in this project: {:?}. \
                    Is localhost:3000 likely correct, or should it be a different endpoint?",
                    context.current_task,
                    self.known_endpoints.keys().collect::<Vec<_>>()
                );
                
                let response = self.llm.generate(&prompt).await?;
                
                if response.contains("incorrect") || response.contains("wrong") {
                    return ValidationAdvice::Question {
                        question: "Are you sure localhost:3000 is the right endpoint?".into(),
                        alternatives: self.known_endpoints.keys().cloned().collect(),
                    };
                }
            }
        }
        
        ValidationAdvice::Proceed
    }
    
    async fn detect_issues(&self, call: &ToolCall, context: &ToolContext) -> Vec<PotentialIssue> {
        if let ToolCall::HttpRequest { url, .. } = call {
            let mut issues = Vec::new();
            
            // Check if endpoint is likely to be available
            if !self.quick_connectivity_check(url).await {
                issues.push(PotentialIssue {
                    severity: Severity::Medium,
                    description: format!("Endpoint {} may not be reachable", url),
                    suggestion: "Consider checking if the service is running".into(),
                });
            }
            
            // Check for HTTP vs HTTPS
            if url.starts_with("http://") && !url.contains("localhost") {
                issues.push(PotentialIssue {
                    severity: Severity::Low,
                    description: "Using HTTP for external endpoint".into(),
                    suggestion: "Consider using HTTPS for security".into(),
                });
            }
            
            issues
        } else {
            vec![]
        }
    }
}
```

#### Package Installation Tool Intelligence

```rust
/// Package installation tool with dependency intelligence  
pub struct PackageInstallTool {
    package_manager: Box<dyn PackageManager>,
}

impl Tool for PackageInstallTool {
    async fn execute(&self, params: ToolParams) -> Result<ToolResult> {
        match params {
            ToolParams::InstallPackage { name, version } => {
                self.package_manager.install(&name, version).await
            }
        }
    }
}

/// Intelligence for package installations
pub struct PackageInstallIntelligence {
    llm: Arc<dyn LLMProvider>,
    project_analyzer: ProjectAnalyzer,
}

impl ToolIntelligence for PackageInstallIntelligence {
    async fn validate_call(&self, call: &ToolCall, context: &ToolContext) -> ValidationAdvice {
        if let ToolCall::InstallPackage { name, .. } = call {
            // Check if installation is actually needed
            if let Some(existing_version) = self.project_analyzer.get_installed_version(name).await {
                return ValidationAdvice::Question {
                    question: format!(
                        "Package {} is already installed (version {}). Still want to install?",
                        name, existing_version
                    ),
                    alternatives: vec![
                        "Check if existing version works".into(),
                        "Update to newer version".into(),
                        "Continue with installation anyway".into(),
                    ],
                };
            }
            
            // Check if this installation is in response to a connection failure
            if context.recent_failures.iter().any(|f| f.is_connection_failure()) {
                let prompt = format!(
                    "About to install {} in response to a connection failure. \
                    The failure was: {}. \
                    Is installation the right solution, or should we check configuration first?",
                    name,
                    context.recent_failures.last().unwrap().message
                );
                
                let response = self.llm.generate(&prompt).await?;
                
                if response.contains("check configuration") || response.contains("not installation") {
                    return ValidationAdvice::Block {
                        reason: "Installation may not solve the connection issue".into(),
                        suggestion: "Check configuration and connectivity first".into(),
                    };
                }
            }
        }
        
        ValidationAdvice::Proceed
    }
    
    async fn suggest_alternatives(&self, call: &ToolCall) -> Vec<AlternativeAction> {
        if let ToolCall::InstallPackage { name, .. } = call {
            let mut alternatives = Vec::new();
            
            // Suggest checking existing installation
            alternatives.push(AlternativeAction {
                description: "Check if package is already available".into(),
                actions: vec![Action::CheckPackage { name: name.clone() }],
            });
            
            // Suggest configuration check if related to connection issues
            if self.context_suggests_connection_issue().await {
                alternatives.push(AlternativeAction {
                    description: "Verify connection configuration instead".into(),
                    actions: vec![
                        Action::CheckConfig { key: "connection_url".into() },
                        Action::TestConnectivity,
                    ],
                });
            }
            
            alternatives
        } else {
            vec![]
        }
    }
}
```

### Configuration System

```rust
/// Tool intelligence configuration
#[derive(Clone, Debug)]
pub struct ToolIntelligenceConfig {
    /// Global enable/disable
    pub enabled: bool,
    
    /// Per-tool intelligence settings
    pub tool_settings: HashMap<String, ToolIntelligenceSettings>,
    
    /// LLM provider configuration
    pub llm_config: LLMConfig,
    
    /// Performance thresholds
    pub performance_limits: PerformanceLimits,
}

#[derive(Clone, Debug)]
pub struct ToolIntelligenceSettings {
    /// Enable intelligence for this tool
    pub enabled: bool,
    
    /// Which intelligence features to enable
    pub features: IntelligenceFeatures,
    
    /// Strictness level
    pub strictness: StrictnessLevel,
    
    /// Maximum LLM call latency before fallback
    pub max_latency: Duration,
}

#[derive(Clone, Debug)]
pub struct IntelligenceFeatures {
    pub validation: bool,
    pub alternatives: bool,
    pub impact_analysis: bool,
    pub issue_detection: bool,
}

/// Tool registry with intelligence capabilities
pub struct IntelligentToolRegistry {
    /// Base tools (always available)
    base_tools: HashMap<String, Box<dyn Tool>>,
    
    /// Intelligence providers (optional)
    intelligence_providers: HashMap<String, Box<dyn ToolIntelligence>>,
    
    /// Configuration
    config: ToolIntelligenceConfig,
    
    /// LLM provider (optional)
    llm_provider: Option<Arc<dyn LLMProvider>>,
}

impl IntelligentToolRegistry {
    /// Get tool with or without intelligence based on config
    pub async fn get_tool(&self, tool_id: &str) -> Option<Box<dyn Tool>> {
        let base_tool = self.base_tools.get(tool_id)?;
        
        // Check if intelligence should be enabled
        if self.should_enable_intelligence(tool_id).await {
            if let Some(intelligence) = self.intelligence_providers.get(tool_id) {
                // Return intelligent version
                Some(Box::new(base_tool.clone().with_intelligence(intelligence.clone())))
            } else {
                // Intelligence requested but not available - return base tool
                Some(base_tool.clone())
            }
        } else {
            // Intelligence disabled - return base tool
            Some(base_tool.clone())
        }
    }
    
    /// Check if intelligence should be enabled for a tool
    async fn should_enable_intelligence(&self, tool_id: &str) -> bool {
        // Global setting
        if !self.config.enabled {
            return false;
        }
        
        // Tool-specific setting
        if let Some(tool_settings) = self.config.tool_settings.get(tool_id) {
            if !tool_settings.enabled {
                return false;
            }
        }
        
        // LLM availability
        if let Some(llm) = &self.llm_provider {
            llm.is_available().await
        } else {
            false
        }
    }
}
```

### Example Usage

```toml
# .patinox/tool-intelligence.toml

[global]
enabled = true

[llm]
provider = "claude"
model = "haiku"  # Lightweight model for tool intelligence
max_tokens = 500
timeout = "5s"

[performance]
max_latency = "2s"
fallback_on_timeout = true

[tools.filesystem]
enabled = true
validation = true
alternatives = true
impact_analysis = false
issue_detection = true
strictness = "medium"

[tools.network]  
enabled = true
validation = true
alternatives = false
impact_analysis = true
issue_detection = true
strictness = "high"

[tools.package_install]
enabled = true
validation = true
alternatives = true
impact_analysis = true
issue_detection = true
strictness = "high"  # Be very careful with installations

[tools.git]
enabled = false  # Disable intelligence for git operations
```

```rust
// Usage example
let registry = IntelligentToolRegistry::new()
    .with_config(config)
    .with_llm_provider(claude_provider);

// Get filesystem tool - may or may not have intelligence
let fs_tool = registry.get_tool("filesystem").await?;

// Execute normally - intelligence is transparent
let result = fs_tool.execute(ToolParams::WriteFile {
    path: "/etc/config.json".into(),
    content: "{}".into(),
}).await;

// If intelligence is enabled and detects issues:
// Result might be: Err(ValidationFailed("LLM detected potential system file overwrite"))
// If intelligence is disabled or unavailable:
// Result: Ok(file written successfully)
```

## Benefits

1. **User Control**: Users choose which tools get intelligence
2. **Performance**: Base tools remain fast, intelligence is additive
3. **Reliability**: Tools work regardless of LLM availability
4. **Contextual**: Intelligence has access to tool-specific context
5. **Adaptive**: Can learn patterns specific to each tool type

## Relationships
- **Parent Nodes:** [elements/supervisory_tool_validation.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/agent_conscience_pattern.md] - coordinates - With agent-level validation
  - [foundation/principles.md] - embodies - User choice and reliability

## Navigation Guidance
- **Access Context:** Reference when implementing intelligent tools
- **Common Next Steps:** Review tool validation integration
- **Related Tasks:** Tool enhancement, LLM integration, user choice architecture
- **Update Patterns:** Update when adding new tool intelligence patterns

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial tool-embedded intelligence design with freedom-without-obligation principle