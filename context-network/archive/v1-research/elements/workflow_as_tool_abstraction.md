# Workflow-as-Tool Abstraction Pattern

## Overview

This document defines how complex multi-step workflows in Patinox can be encapsulated and exposed as simple tools. This abstraction allows workflows to be composed, reused, and integrated seamlessly while hiding their internal complexity from consumers.

## Core Concept

A workflow is a directed graph of operations that accomplishes a complex task. By wrapping workflows as tools, we achieve:

1. **Composability**: Workflows can contain other workflows
2. **Reusability**: Complex patterns become simple building blocks
3. **Abstraction**: Internal complexity is hidden from consumers
4. **Versioning**: Workflows can evolve independently
5. **Portability**: Workflows appear as standard tools to any system

## Workflow Definition

### Basic Workflow Structure

```rust
/// Core workflow trait that all workflows implement
pub trait Workflow: Send + Sync {
    /// Unique identifier for this workflow
    fn id(&self) -> &WorkflowId;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Description of what this workflow does
    fn description(&self) -> &str;
    
    /// Input schema for the workflow
    fn input_schema(&self) -> &Schema;
    
    /// Output schema for the workflow
    fn output_schema(&self) -> &Schema;
    
    /// Execute the workflow
    async fn execute(&self, input: Value, context: Context) -> Result<Value>;
    
    /// Get workflow metadata
    fn metadata(&self) -> WorkflowMetadata;
}

/// Metadata about a workflow
pub struct WorkflowMetadata {
    pub version: Version,
    pub author: String,
    pub tags: Vec<String>,
    pub estimated_duration: Option<Duration>,
    pub resource_requirements: ResourceRequirements,
    pub idempotent: bool,
    pub retry_policy: RetryPolicy,
}
```

### Workflow Implementation

```rust
/// A concrete workflow implementation
pub struct DataProcessingWorkflow {
    id: WorkflowId,
    steps: Vec<Box<dyn WorkflowStep>>,
    error_handler: Box<dyn ErrorHandler>,
    state_manager: StateManager,
}

impl DataProcessingWorkflow {
    pub fn builder() -> WorkflowBuilder {
        WorkflowBuilder::new()
    }
}

/// Builder for constructing workflows
pub struct WorkflowBuilder {
    steps: Vec<Box<dyn WorkflowStep>>,
    error_handling: ErrorStrategy,
    parallelism: ParallelismConfig,
}

impl WorkflowBuilder {
    pub fn add_step<S: WorkflowStep + 'static>(mut self, step: S) -> Self {
        self.steps.push(Box::new(step));
        self
    }
    
    pub fn parallel_group<F>(mut self, f: F) -> Self 
    where
        F: FnOnce(ParallelGroupBuilder) -> ParallelGroupBuilder
    {
        let group = f(ParallelGroupBuilder::new()).build();
        self.steps.push(Box::new(group));
        self
    }
    
    pub fn conditional<C, T, F>(mut self, condition: C, then: T, otherwise: F) -> Self
    where
        C: Condition + 'static,
        T: WorkflowStep + 'static,
        F: WorkflowStep + 'static,
    {
        self.steps.push(Box::new(ConditionalStep {
            condition: Box::new(condition),
            then_branch: Box::new(then),
            else_branch: Box::new(otherwise),
        }));
        self
    }
    
    pub fn build(self) -> DataProcessingWorkflow {
        DataProcessingWorkflow {
            id: WorkflowId::generate(),
            steps: self.steps,
            error_handler: self.error_handling.into_handler(),
            state_manager: StateManager::new(),
        }
    }
}
```

## Tool Wrapper Pattern

### Converting Workflow to Tool

```rust
/// Wrapper that makes a workflow appear as a tool
pub struct WorkflowTool<W: Workflow> {
    workflow: W,
    adapter: ProtocolAdapter,
}

impl<W: Workflow> Tool for WorkflowTool<W> {
    fn name(&self) -> &str {
        self.workflow.name()
    }
    
    fn description(&self) -> &str {
        self.workflow.description()
    }
    
    fn parameters(&self) -> &Schema {
        self.workflow.input_schema()
    }
    
    fn returns(&self) -> &Schema {
        self.workflow.output_schema()
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        // Adapt tool parameters to workflow input
        let workflow_input = self.adapter.adapt_input(params)?;
        
        // Create execution context
        let context = Context::from_current();
        
        // Execute workflow
        let result = self.workflow.execute(workflow_input, context).await?;
        
        // Adapt workflow output to tool format
        self.adapter.adapt_output(result)
    }
}

/// Extension trait to convert any workflow to a tool
pub trait WorkflowExt: Workflow {
    fn as_tool(self) -> WorkflowTool<Self> 
    where 
        Self: Sized 
    {
        WorkflowTool {
            workflow: self,
            adapter: ProtocolAdapter::default(),
        }
    }
}

impl<W: Workflow> WorkflowExt for W {}
```

### Workflow Composition

Workflows can contain other workflows:

```rust
/// A workflow step that executes another workflow
pub struct SubWorkflowStep {
    workflow: Box<dyn Workflow>,
    input_mapping: InputMapping,
    output_mapping: OutputMapping,
}

impl WorkflowStep for SubWorkflowStep {
    async fn execute(&self, input: StepInput) -> Result<StepOutput> {
        // Map input from parent workflow to sub-workflow
        let sub_input = self.input_mapping.map(input)?;
        
        // Execute sub-workflow
        let sub_output = self.workflow.execute(
            sub_input,
            input.context.clone()
        ).await?;
        
        // Map output back to parent workflow
        self.output_mapping.map(sub_output)
    }
}

/// Example of nested workflows
pub fn create_complex_workflow() -> impl Workflow {
    WorkflowBuilder::new()
        .add_step(DataValidationStep::new())
        .add_step(SubWorkflowStep {
            workflow: Box::new(create_enrichment_workflow()),
            input_mapping: InputMapping::direct(),
            output_mapping: OutputMapping::merge(),
        })
        .parallel_group(|group| {
            group
                .add(SubWorkflowStep {
                    workflow: Box::new(create_analysis_workflow()),
                    input_mapping: InputMapping::field("data"),
                    output_mapping: OutputMapping::field("analysis"),
                })
                .add(SubWorkflowStep {
                    workflow: Box::new(create_summary_workflow()),
                    input_mapping: InputMapping::field("data"),
                    output_mapping: OutputMapping::field("summary"),
                })
        })
        .add_step(ResultAggregationStep::new())
        .build()
}
```

## State Management

Workflows maintain state across executions:

```rust
/// Workflow state management
pub struct WorkflowState {
    pub workflow_id: WorkflowId,
    pub execution_id: ExecutionId,
    pub status: WorkflowStatus,
    pub current_step: Option<StepId>,
    pub step_outputs: HashMap<StepId, Value>,
    pub context: Context,
    pub checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Debug)]
pub enum WorkflowStatus {
    NotStarted,
    Running { 
        started_at: DateTime<Utc>,
        progress: Progress,
    },
    Paused {
        paused_at: DateTime<Utc>,
        reason: String,
    },
    Completed {
        completed_at: DateTime<Utc>,
        output: Value,
    },
    Failed {
        failed_at: DateTime<Utc>,
        error: WorkflowError,
        recoverable: bool,
    },
}

/// State manager for workflow execution
pub struct StateManager {
    storage: Box<dyn StateStorage>,
}

impl StateManager {
    pub async fn save_checkpoint(&self, state: &WorkflowState) -> Result<()> {
        let checkpoint = Checkpoint {
            id: CheckpointId::generate(),
            timestamp: Utc::now(),
            state: state.clone(),
        };
        
        self.storage.save_checkpoint(checkpoint).await
    }
    
    pub async fn restore_from_checkpoint(
        &self,
        checkpoint_id: CheckpointId,
    ) -> Result<WorkflowState> {
        self.storage.load_checkpoint(checkpoint_id).await
    }
    
    pub async fn get_latest_checkpoint(
        &self,
        workflow_id: WorkflowId,
    ) -> Result<Option<Checkpoint>> {
        self.storage.get_latest(workflow_id).await
    }
}
```

## Resumable Workflows

Workflows can be paused and resumed:

```rust
/// Resumable workflow execution
pub struct ResumableWorkflow<W: Workflow> {
    workflow: W,
    state_manager: StateManager,
    resume_strategy: ResumeStrategy,
}

impl<W: Workflow> ResumableWorkflow<W> {
    pub async fn execute_resumable(
        &self,
        input: Value,
        context: Context,
    ) -> Result<Value> {
        // Check for existing execution
        if let Some(checkpoint) = self.state_manager
            .get_latest_checkpoint(self.workflow.id().clone())
            .await? 
        {
            // Resume from checkpoint
            self.resume_from_checkpoint(checkpoint, context).await
        } else {
            // Start fresh execution
            self.execute_fresh(input, context).await
        }
    }
    
    async fn resume_from_checkpoint(
        &self,
        checkpoint: Checkpoint,
        context: Context,
    ) -> Result<Value> {
        let mut state = checkpoint.state;
        
        match self.resume_strategy {
            ResumeStrategy::FromLastStep => {
                // Continue from where we left off
                self.execute_from_step(state, context).await
            }
            ResumeStrategy::RetryLastStep => {
                // Retry the step that failed
                if let Some(step_id) = state.current_step {
                    state.step_outputs.remove(&step_id);
                }
                self.execute_from_step(state, context).await
            }
            ResumeStrategy::FromBeginning => {
                // Start over but with saved context
                self.execute_fresh(
                    state.context.get("original_input").unwrap(),
                    context
                ).await
            }
        }
    }
}

#[derive(Clone)]
pub enum ResumeStrategy {
    FromLastStep,
    RetryLastStep,
    FromBeginning,
}
```

## Workflow Versioning

Support for multiple workflow versions:

```rust
/// Versioned workflow registry
pub struct WorkflowRegistry {
    workflows: HashMap<WorkflowId, VersionedWorkflow>,
}

pub struct VersionedWorkflow {
    versions: BTreeMap<Version, Box<dyn Workflow>>,
    default_version: Version,
    migration_strategies: HashMap<(Version, Version), MigrationStrategy>,
}

impl VersionedWorkflow {
    pub fn get_version(&self, version: Option<Version>) -> &dyn Workflow {
        let v = version.unwrap_or(self.default_version.clone());
        self.versions.get(&v)
            .map(|w| w.as_ref())
            .unwrap_or_else(|| {
                // Fall back to default
                self.versions.get(&self.default_version)
                    .map(|w| w.as_ref())
                    .expect("Default version must exist")
            })
    }
    
    pub async fn migrate_data(
        &self,
        data: Value,
        from_version: Version,
        to_version: Version,
    ) -> Result<Value> {
        let key = (from_version.clone(), to_version.clone());
        
        if let Some(strategy) = self.migration_strategies.get(&key) {
            strategy.migrate(data).await
        } else if from_version == to_version {
            Ok(data)
        } else {
            Err(Error::NoMigrationPath { from: from_version, to: to_version })
        }
    }
}
```

## Workflow Monitoring

Observability for workflow execution:

```rust
/// Workflow monitoring and telemetry
pub struct WorkflowMonitor {
    metrics: MetricsCollector,
    tracer: Tracer,
    event_bus: EventBus,
}

impl WorkflowMonitor {
    pub async fn start_execution(&self, workflow: &dyn Workflow) -> ExecutionSpan {
        let span = self.tracer.start_span("workflow.execute")
            .with_attribute("workflow.id", workflow.id())
            .with_attribute("workflow.name", workflow.name())
            .with_attribute("workflow.version", workflow.metadata().version);
        
        self.metrics.increment("workflow.executions.started");
        
        self.event_bus.publish(WorkflowEvent::Started {
            workflow_id: workflow.id().clone(),
            timestamp: Utc::now(),
        }).await;
        
        span
    }
    
    pub async fn record_step_completion(
        &self,
        workflow_id: &WorkflowId,
        step_id: &StepId,
        duration: Duration,
        success: bool,
    ) {
        self.metrics.record_histogram(
            "workflow.step.duration",
            duration.as_secs_f64(),
            &[
                ("workflow_id", workflow_id.as_str()),
                ("step_id", step_id.as_str()),
                ("success", &success.to_string()),
            ],
        );
        
        if success {
            self.metrics.increment("workflow.steps.completed");
        } else {
            self.metrics.increment("workflow.steps.failed");
        }
    }
}
```

## Dynamic Workflow Generation

Workflows can be generated dynamically:

```rust
/// Dynamic workflow generator
pub struct DynamicWorkflowGenerator {
    template_engine: TemplateEngine,
    step_library: StepLibrary,
}

impl DynamicWorkflowGenerator {
    pub async fn generate_from_description(
        &self,
        description: &str,
    ) -> Result<Box<dyn Workflow>> {
        // Use LLM to understand the workflow requirements
        let requirements = self.analyze_requirements(description).await?;
        
        // Select appropriate steps from library
        let steps = self.select_steps(&requirements)?;
        
        // Generate workflow configuration
        let config = self.generate_config(&requirements, &steps)?;
        
        // Build the workflow
        self.build_workflow(config, steps)
    }
    
    async fn analyze_requirements(&self, description: &str) -> Result<Requirements> {
        // Use LLM to extract structured requirements
        let prompt = format!(
            "Analyze this workflow description and extract requirements:\n{}",
            description
        );
        
        let response = self.llm.complete(&prompt).await?;
        serde_json::from_str(&response)
    }
    
    fn select_steps(&self, requirements: &Requirements) -> Result<Vec<StepTemplate>> {
        let mut steps = Vec::new();
        
        for capability in &requirements.capabilities {
            if let Some(step) = self.step_library.find_by_capability(capability) {
                steps.push(step);
            } else {
                return Err(Error::MissingCapability(capability.clone()));
            }
        }
        
        Ok(steps)
    }
}
```

## Workflow as Tool Examples

### Example 1: Data Pipeline Workflow

```rust
pub fn create_data_pipeline() -> impl Workflow {
    WorkflowBuilder::new()
        .add_step(DataIngestionStep::new())
        .add_step(DataValidationStep::new())
        .parallel_group(|group| {
            group
                .add(DataCleaningStep::new())
                .add(DataEnrichmentStep::new())
        })
        .add_step(DataTransformationStep::new())
        .conditional(
            DataQualityCheck::new(),
            DataStorageStep::new(),
            DataRejectionStep::new(),
        )
        .build()
}

// Expose as tool
let pipeline_tool = create_data_pipeline().as_tool();
```

### Example 2: Document Processing Workflow

```rust
pub struct DocumentProcessingWorkflow {
    ocr_step: OCRStep,
    extraction_step: InformationExtractionStep,
    validation_step: ValidationStep,
    storage_step: StorageStep,
}

impl Workflow for DocumentProcessingWorkflow {
    async fn execute(&self, input: Value, context: Context) -> Result<Value> {
        // Chain steps with error handling
        let document = input.as_document()?;
        
        let text = self.ocr_step.process(document).await
            .map_err(|e| WorkflowError::Step("OCR", e))?;
        
        let extracted = self.extraction_step.extract(text).await
            .map_err(|e| WorkflowError::Step("Extraction", e))?;
        
        let validated = self.validation_step.validate(extracted).await
            .map_err(|e| WorkflowError::Step("Validation", e))?;
        
        let stored = self.storage_step.store(validated).await
            .map_err(|e| WorkflowError::Step("Storage", e))?;
        
        Ok(json!({
            "document_id": stored.id,
            "extracted_data": stored.data,
            "metadata": stored.metadata,
        }))
    }
}
```

### Example 3: Multi-Agent Coordination Workflow

```rust
pub struct MultiAgentWorkflow {
    agents: Vec<AgentId>,
    coordination_strategy: CoordinationStrategy,
}

impl Workflow for MultiAgentWorkflow {
    async fn execute(&self, input: Value, context: Context) -> Result<Value> {
        match self.coordination_strategy {
            CoordinationStrategy::Sequential => {
                let mut result = input;
                for agent_id in &self.agents {
                    result = self.execute_agent(agent_id, result).await?;
                }
                Ok(result)
            }
            CoordinationStrategy::Parallel => {
                let futures = self.agents.iter()
                    .map(|id| self.execute_agent(id, input.clone()));
                
                let results = futures::future::join_all(futures).await;
                self.merge_results(results)
            }
            CoordinationStrategy::Voting => {
                let futures = self.agents.iter()
                    .map(|id| self.execute_agent(id, input.clone()));
                
                let results = futures::future::join_all(futures).await;
                self.vote_on_results(results)
            }
        }
    }
}
```

## Best Practices

1. **Single Responsibility**: Each workflow should have a clear, single purpose
2. **Idempotency**: Design workflows to be safely retryable
3. **Error Boundaries**: Define clear error handling at workflow boundaries
4. **State Persistence**: Save state at critical points for resumability
5. **Version Compatibility**: Maintain backward compatibility when possible
6. **Monitoring**: Instrument workflows for observability
7. **Documentation**: Clearly document inputs, outputs, and side effects

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/protocol_based_exposure.md] - exposes - Workflows through protocols
  - [elements/resumable_workflows.md] - implements - Resumption capabilities
  - [elements/interruptible_agent_loops.md] - coordinates - With agent execution

## Navigation Guidance
- **Access Context:** Reference when designing complex multi-step operations
- **Common Next Steps:** Review CLI exposure or agent conscience patterns
- **Related Tasks:** Workflow design, tool creation, composition patterns
- **Update Patterns:** Update when adding new workflow patterns or capabilities

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial workflow-as-tool abstraction pattern design