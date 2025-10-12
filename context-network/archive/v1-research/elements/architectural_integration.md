# Architectural Integration: Unified Design Patterns

## Overview

This document demonstrates how all the architectural patterns and concepts in Patinox work together to create a cohesive, flexible, and powerful agent framework. It shows the integration points between dependency injection, protocols, workflows, CLI exposure, configuration, and conscience patterns.

## Integrated Architecture Flow

### 1. System Bootstrap

The system starts with configuration and dependency injection working together:

```rust
// 1. Load configuration with devcontainer awareness
let config = CLIConfig::load()
    .with_dotfolder_priority()  // Local .patinox folder
    .with_env_overrides()        // Environment variables
    .await?;

// 2. Setup dependency injection with freedom-without-obligation
let dependencies = DependencyBuilder::new()
    .with_defaults()             // Works out of the box
    .override_from_config(&config)  // Optional overrides
    .build()?;

// 3. Initialize conscience layer
let conscience = ConscienceProfile::from_config(&config)
    .unwrap_or(ConscienceProfile::Standard)
    .to_conscience();

// 4. Create agent with all layers
let agent = Agent::builder()
    .with_dependencies(dependencies)
    .with_conscience(conscience)
    .with_paradigm(AgentParadigm::ReAct)  // or others
    .build()?;
```

### 2. Protocol and CLI Integration

Agents are exposed through multiple interfaces simultaneously:

```rust
// CLI exposure for local operation
let cli_server = CLIServer::new(agent.clone())
    .with_repl_support()
    .with_pipeline_mode()
    .with_offline_first();

// Protocol exposure for connectivity
let protocol_server = ProtocolServer::new(agent.clone())
    .with_mcp()      // Model Context Protocol
    .with_a2a()      // Agent-to-Agent
    .with_rest()     // Web API
    .with_grpc();    // High-performance

// Connection manager for hybrid operation
let connection_manager = ConnectionManager::new()
    .prefer_local()  // CLI philosophy: local-first
    .opportunistic_remote()  // Connect when available
    .with_discovery(ServiceDiscovery::mdns());

// Unified execution
let executor = UnifiedExecutor::new()
    .add_interface(cli_server)
    .add_interface(protocol_server)
    .with_connection_manager(connection_manager);
```

### 3. Workflow and Tool Composition

Workflows become tools, tools expose through protocols:

```rust
// Create a complex workflow
let data_pipeline = WorkflowBuilder::new()
    .add_step(DataIngestion::new())
    .add_step(DataValidation::with_conscience())  // Conscience integration
    .parallel_group(|group| {
        group
            .add(DataCleaning::new())
            .add(DataEnrichment::with_remote_tools())  // Protocol integration
    })
    .add_step(DataStorage::new())
    .build();

// Wrap workflow as tool
let pipeline_tool = data_pipeline.as_tool();

// Expose tool through MCP
mcp_server.expose_tool(pipeline_tool);

// Make available to CLI
cli_registry.register_tool(pipeline_tool);

// Use in agent with dependency injection
let agent = Agent::builder()
    .with_tool(pipeline_tool)  // Injected dependency
    .build()?;
```

### 4. Configuration Cascade Example

Configuration flows through all layers:

```rust
// Global configuration from .patinox/config.toml
[agent.researcher]
paradigm = "tree-of-thoughts"
conscience = "thorough"
max_depth = 5

[protocols.mcp]
enabled = true
port = 8080

[cli]
offline_mode = false
repl_history = ".patinox/history"

// Environment override
PATINOX_AGENT_RESEARCHER_PARADIGM=reflexion

// Runtime override
let agent = Agent::get("researcher")
    .with_paradigm_override(Paradigm::ChainOfThought)  // Highest priority
    .execute(task)
    .await?;
```

### 5. Conscience Integration Points

The conscience pattern integrates across all components:

```rust
// In workflow steps
pub struct ValidatedStep<S: WorkflowStep> {
    step: S,
    conscience: Box<dyn Conscience>,
}

impl<S: WorkflowStep> WorkflowStep for ValidatedStep<S> {
    async fn execute(&self, input: StepInput) -> Result<StepOutput> {
        // Pre-execution conscience check
        let decision = self.conscience.evaluate_action(
            &self.step.to_action(),
            &input.context
        ).await;
        
        if let ConscienceDecision::Reject { reason } = decision {
            return Err(ConscienceError::Rejected(reason));
        }
        
        // Execute step
        let result = self.step.execute(input).await;
        
        // Post-execution review
        let review = self.conscience.review_result(&result, &context).await;
        
        match review {
            ConscienceReview::Unacceptable { reasons, .. } => {
                Err(ConscienceError::Unacceptable(reasons))
            }
            _ => result
        }
    }
}

// In protocol handlers
impl MCPServer {
    async fn execute_tool_with_conscience(
        &self,
        tool: &str,
        params: Value,
    ) -> Result<Value> {
        // Check conscience before tool execution
        if let Some(conscience) = &self.conscience {
            let action = Action::ToolExecution { tool, params: params.clone() };
            conscience.validate_action(&action).await?;
        }
        
        let result = self.execute_tool(tool, params).await;
        
        // Review result
        if let Some(conscience) = &self.conscience {
            conscience.review_tool_result(tool, &result).await?;
        }
        
        result
    }
}
```

### 6. Hybrid Local/Remote Execution

Seamless switching between local CLI and remote protocols:

```rust
pub struct HybridExecutor {
    local_agent: Box<dyn Agent>,
    remote_clients: HashMap<AgentId, Box<dyn RemoteAgent>>,
    connection_mode: Arc<RwLock<ConnectionMode>>,
}

impl HybridExecutor {
    pub async fn execute(&self, task: Task) -> Result<Value> {
        let mode = self.connection_mode.read().await;
        
        match *mode {
            ConnectionMode::Offline => {
                // Pure local execution through CLI
                self.local_agent.execute(task).await
            }
            
            ConnectionMode::Hybrid { ref preference } => {
                // Try preferred, fall back to other
                match preference {
                    ConnectionPreference::PreferLocal => {
                        self.local_agent.execute(task.clone()).await
                            .or_else(|_| self.execute_remote(task).await)
                    }
                    ConnectionPreference::PreferRemote => {
                        self.execute_remote(task.clone()).await
                            .or_else(|_| self.local_agent.execute(task).await)
                    }
                }
            }
            
            ConnectionMode::Federated => {
                // Distribute across multiple agents
                self.federated_execution(task).await
            }
        }
    }
    
    async fn execute_remote(&self, task: Task) -> Result<Value> {
        // Try different protocols in order
        if let Some(a2a_client) = self.get_a2a_client(&task.agent_hint) {
            return a2a_client.execute(task).await;
        }
        
        if let Some(mcp_client) = self.get_mcp_client(&task.tool_hint) {
            return mcp_client.execute_tool(task.into()).await;
        }
        
        // Fall back to REST
        self.rest_client.post("/execute", &task).await
    }
}
```

## Complete Example: Research Assistant

Here's how all patterns work together in a real scenario:

```rust
/// Research assistant that demonstrates all patterns
pub async fn create_research_assistant() -> Result<Box<dyn Agent>> {
    // 1. Configuration (with dotfolder and env)
    let config = ResearchConfig::load_cascade().await?;
    
    // 2. Dependency Injection (freedom without obligation)
    let deps = ResearchDependencies::builder()
        .with_default_llm()  // Can override
        .with_default_tools()  // Can override
        .with_custom_validator(AcademicValidator::new())  // Custom addition
        .build()?;
    
    // 3. Conscience (internal validation)
    let conscience = ResearchConscience::new()
        .with_citation_checking()
        .with_fact_verification()
        .with_source_validation();
    
    // 4. Workflow as Tool
    let research_workflow = WorkflowBuilder::new()
        .add_step(TopicAnalysis::new())
        .add_step(SourceDiscovery::new())
        .parallel_group(|g| g
            .add(AcademicSearch::new())
            .add(WebSearch::new())
            .add(DatabaseQuery::new())
        )
        .add_step(SourceValidation::with_conscience(conscience.clone()))
        .add_step(InformationSynthesis::new())
        .add_step(CitationGeneration::new())
        .build()
        .as_tool();  // Convert to tool
    
    // 5. Agent with Paradigm
    let agent = ResearchAgent::builder()
        .with_dependencies(deps)
        .with_conscience(conscience)
        .with_paradigm(AgentParadigm::TreeOfThoughts)  // For exploration
        .with_tool(research_workflow)
        .interruptible()  // Can be paused/resumed
        .build()?;
    
    // 6. Protocol Exposure
    let protocol_layer = ProtocolLayer::new()
        .expose_via_mcp(agent.clone())
        .expose_via_a2a(agent.clone())
        .expose_via_graphql(agent.clone());
    
    // 7. CLI Exposure
    let cli_layer = CLILayer::new()
        .with_repl(agent.clone())
        .with_pipeline_support()
        .with_offline_mode()
        .with_local_filesystem_access();
    
    // 8. Connection Management
    let connection_mgr = ConnectionManager::new()
        .discover_peers()  // Find other research agents
        .prefer_local()    // Use local resources first
        .cache_remote();   // Cache remote results
    
    // Return composed agent
    Ok(Box::new(
        ComposedAgent::new()
            .core(agent)
            .protocols(protocol_layer)
            .cli(cli_layer)
            .connections(connection_mgr)
    ))
}

// Usage through CLI
$ patinox agent research --topic "quantum computing" --depth 3

// Usage through REPL
$ patinox repl --agent research
> find papers on quantum error correction
> validate sources with academic databases
> generate bibliography

// Usage through protocol
let client = MCPClient::connect("localhost:8080");
let result = client.execute_tool("research", json!({
    "topic": "quantum computing",
    "depth": 3
})).await?;

// Usage in workflow
let analysis_workflow = WorkflowBuilder::new()
    .add_step(ResearchStep::new(research_agent))
    .add_step(AnalysisStep::new())
    .build();
```

## Integration Benefits

### 1. Composability
Every component can be composed with others:
- Workflows contain workflows
- Tools expose through protocols
- Agents use other agents
- Conscience wraps any execution

### 2. Flexibility
Multiple options at every level:
- DI allows component swapping
- Protocols enable various connections
- CLI works offline or online
- Configuration cascades appropriately

### 3. Consistency
Patterns apply uniformly:
- Same conscience rules everywhere
- Configuration works identically
- Protocols abstract uniformly
- DI pattern is consistent

### 4. Locality
Local-first with optional connectivity:
- CLI doesn't require network
- Dotfolders provide local config
- Offline mode is default
- Remote is enhancement, not requirement

### 5. Evolution
System can grow and adapt:
- New protocols can be added
- Workflows evolve independently
- Conscience learns patterns
- Configuration extends cleanly

## Best Practices for Integration

1. **Start Simple**: Use defaults, add complexity as needed
2. **Layer Appropriately**: Each pattern has its place
3. **Maintain Boundaries**: Keep concerns separated
4. **Document Connections**: Make integration points clear
5. **Test Combinations**: Verify patterns work together
6. **Profile Performance**: Monitor integration overhead
7. **Version Carefully**: Maintain compatibility

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/dependency_injection_philosophy.md] - integrates
  - [elements/protocol_based_exposure.md] - integrates
  - [elements/workflow_as_tool_abstraction.md] - integrates
  - [elements/cli_agent_exposure.md] - integrates
  - [elements/configuration_strategy.md] - integrates
  - [elements/agent_conscience_pattern.md] - integrates

## Navigation Guidance
- **Access Context:** Reference when understanding system integration
- **Common Next Steps:** Implement specific integration patterns
- **Related Tasks:** System design, architecture review, implementation
- **Update Patterns:** Update when integration patterns change

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial architectural integration documentation showing unified patterns