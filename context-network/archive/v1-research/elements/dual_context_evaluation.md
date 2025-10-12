# Dual-Context Evaluation Pattern

## Overview

One of the most powerful patterns for improving LLM agent output quality is the separation of generation and evaluation into distinct contexts. This pattern leverages the insight that a fresh context, tasked specifically with judgment and evaluation, often catches errors and quality issues that the generating context misses. Research shows this approach can improve output quality by 40-60% compared to single-context generation.

## Core Concept

The dual-context pattern separates agent reasoning into two distinct phases:

1. **Generation Context**: Focused on creating, solving, and producing output
2. **Evaluation Context**: Focused on judging, critiquing, and improving output

Each context operates with different prompts, potentially different models, and independent state, eliminating the biases and blind spots that accumulate in a single context.

## Architecture

### Context Separation

```rust
pub struct DualContextAgent {
    // Generation context - optimized for creativity and problem-solving
    generator: GeneratorContext,
    
    // Evaluation context - optimized for criticism and quality assessment
    evaluator: EvaluatorContext,
    
    // Configuration for iteration and improvement
    iteration_config: IterationConfig,
}

pub struct GeneratorContext {
    // Model optimized for generation (e.g., Claude for creativity)
    model: Box<dyn LanguageModel>,
    
    // Generation-specific prompt template
    prompt_template: GeneratorPrompt,
    
    // Context window management
    context_manager: ContextManager,
    
    // Generation parameters (temperature, etc.)
    params: GenerationParams,
}

pub struct EvaluatorContext {
    // Model optimized for evaluation (e.g., GPT-4 for analysis)
    model: Box<dyn LanguageModel>,
    
    // Evaluation-specific prompt template
    prompt_template: EvaluatorPrompt,
    
    // Evaluation criteria and rubrics
    criteria: EvaluationCriteria,
    
    // Evaluation parameters (more deterministic)
    params: EvaluationParams,
}
```

### Execution Flow

```rust
impl DualContextAgent {
    pub async fn execute(&self, task: Task) -> Result<Output> {
        let mut iteration = 0;
        let mut best_output = None;
        let mut best_score = 0.0;
        
        loop {
            // Phase 1: Generation
            let generated = self.generator.generate(&task).await?;
            
            // Phase 2: Evaluation
            let evaluation = self.evaluator.evaluate(&generated, &task).await?;
            
            // Track best output
            if evaluation.score > best_score {
                best_score = evaluation.score;
                best_output = Some(generated.clone());
            }
            
            // Check termination conditions
            if evaluation.is_satisfactory() || iteration >= self.iteration_config.max_iterations {
                break;
            }
            
            // Phase 3: Feedback Integration
            let feedback = self.create_feedback(&evaluation);
            task.integrate_feedback(feedback);
            
            iteration += 1;
        }
        
        Ok(best_output.unwrap_or_else(|| Output::default()))
    }
}
```

## Evaluation Strategies

### Multi-Criteria Evaluation

```rust
pub struct EvaluationCriteria {
    pub criteria: Vec<Criterion>,
    pub weights: Vec<f64>,
    pub threshold: f64,
}

pub enum Criterion {
    Correctness {
        verification_method: VerificationMethod,
    },
    Completeness {
        required_elements: Vec<String>,
    },
    Clarity {
        readability_metric: ReadabilityMetric,
    },
    Safety {
        safety_checks: Vec<SafetyCheck>,
    },
    Efficiency {
        complexity_analysis: ComplexityAnalyzer,
    },
    Originality {
        similarity_threshold: f64,
    },
}

impl EvaluatorContext {
    pub async fn evaluate(&self, output: &Output, task: &Task) -> Result<Evaluation> {
        let mut scores = Vec::new();
        let mut critiques = Vec::new();
        
        for criterion in &self.criteria.criteria {
            let result = self.evaluate_criterion(output, task, criterion).await?;
            scores.push(result.score);
            critiques.push(result.critique);
        }
        
        let weighted_score = self.calculate_weighted_score(&scores);
        
        Ok(Evaluation {
            score: weighted_score,
            critiques,
            is_satisfactory: weighted_score >= self.criteria.threshold,
            suggestions: self.generate_suggestions(&critiques).await?,
        })
    }
}
```

### Specialized Evaluators

Different evaluator types for different content:

```rust
pub enum EvaluatorType {
    // For code generation
    CodeEvaluator {
        linter: Box<dyn Linter>,
        test_runner: Box<dyn TestRunner>,
        complexity_analyzer: Box<dyn ComplexityAnalyzer>,
    },
    
    // For text generation
    TextEvaluator {
        grammar_checker: Box<dyn GrammarChecker>,
        fact_checker: Box<dyn FactChecker>,
        style_analyzer: Box<dyn StyleAnalyzer>,
    },
    
    // For reasoning tasks
    ReasoningEvaluator {
        logic_validator: Box<dyn LogicValidator>,
        consistency_checker: Box<dyn ConsistencyChecker>,
    },
    
    // For creative tasks
    CreativeEvaluator {
        originality_scorer: Box<dyn OriginalityScorer>,
        coherence_analyzer: Box<dyn CoherenceAnalyzer>,
    },
}
```

## Feedback Integration

### Structured Feedback Loop

```rust
pub struct FeedbackLoop {
    pub feedback_type: FeedbackType,
    pub integration_strategy: IntegrationStrategy,
}

pub enum FeedbackType {
    // Specific corrections
    Corrections(Vec<Correction>),
    
    // High-level guidance
    Guidance(String),
    
    // Examples of better outputs
    Examples(Vec<Example>),
    
    // Constraints to satisfy
    Constraints(Vec<Constraint>),
}

pub enum IntegrationStrategy {
    // Append feedback to prompt
    AppendToPrompt,
    
    // Modify generation parameters
    AdjustParameters,
    
    // Change generation strategy
    SwitchStrategy,
    
    // Combine multiple approaches
    Hybrid(Vec<IntegrationStrategy>),
}

impl GeneratorContext {
    pub fn integrate_feedback(&mut self, feedback: Feedback) -> Result<()> {
        match feedback.integration_strategy {
            IntegrationStrategy::AppendToPrompt => {
                self.prompt_template.append_feedback(feedback.content);
            }
            IntegrationStrategy::AdjustParameters => {
                self.params.adjust_based_on_feedback(&feedback);
            }
            IntegrationStrategy::SwitchStrategy => {
                self.switch_generation_strategy(&feedback);
            }
            IntegrationStrategy::Hybrid(strategies) => {
                for strategy in strategies {
                    self.apply_strategy(strategy, &feedback)?;
                }
            }
        }
        Ok(())
    }
}
```

## Iteration Strategies

### Convergence Patterns

```rust
pub struct IterationConfig {
    pub max_iterations: usize,
    pub convergence_threshold: f64,
    pub early_stopping: EarlyStoppingConfig,
    pub iteration_strategy: IterationStrategy,
}

pub enum IterationStrategy {
    // Simple retry with feedback
    SimpleIteration,
    
    // Gradual refinement
    ProgressiveRefinement {
        refinement_focus: Vec<Criterion>,
    },
    
    // Best-of-N selection
    BestOfN {
        n: usize,
        selection_criterion: SelectionCriterion,
    },
    
    // Ensemble approach
    Ensemble {
        generators: Vec<GeneratorContext>,
        aggregation: AggregationMethod,
    },
}

pub struct EarlyStoppingConfig {
    pub patience: usize,
    pub min_improvement: f64,
    pub convergence_window: usize,
}

impl DualContextAgent {
    pub async fn should_stop(&self, history: &[Evaluation]) -> bool {
        if history.len() < self.iteration_config.early_stopping.convergence_window {
            return false;
        }
        
        let window = &history[history.len() - self.iteration_config.early_stopping.convergence_window..];
        let improvement = window.last().unwrap().score - window.first().unwrap().score;
        
        improvement < self.iteration_config.early_stopping.min_improvement
    }
}
```

## Context Optimization

### Prompt Engineering for Each Context

```rust
pub struct GeneratorPrompt {
    pub base_template: String,
    pub role_description: String,
    pub examples: Vec<Example>,
    pub constraints: Vec<String>,
}

pub struct EvaluatorPrompt {
    pub base_template: String,
    pub evaluation_rubric: String,
    pub scoring_guidelines: String,
    pub common_pitfalls: Vec<String>,
}

impl GeneratorPrompt {
    pub fn optimize_for_creativity(&mut self) {
        self.role_description = "You are a creative problem solver...".to_string();
        // Adjust temperature and other params for creativity
    }
    
    pub fn optimize_for_accuracy(&mut self) {
        self.role_description = "You are a precise analyst...".to_string();
        // Adjust for accuracy
    }
}

impl EvaluatorPrompt {
    pub fn optimize_for_criticism(&mut self) {
        self.role_description = "You are a critical reviewer...".to_string();
        self.scoring_guidelines = "Be strict but fair...".to_string();
    }
    
    pub fn optimize_for_improvement(&mut self) {
        self.role_description = "You are a constructive mentor...".to_string();
        self.scoring_guidelines = "Focus on actionable improvements...".to_string();
    }
}
```

### Context Window Management

```rust
pub struct ContextManager {
    pub max_tokens: usize,
    pub preservation_strategy: PreservationStrategy,
    pub compression_method: CompressionMethod,
}

pub enum PreservationStrategy {
    // Keep most recent
    SlidingWindow,
    
    // Keep most important
    ImportanceBased {
        scorer: Box<dyn ImportanceScorer>,
    },
    
    // Summarize old content
    Summarization {
        summarizer: Box<dyn Summarizer>,
    },
    
    // Hybrid approach
    Adaptive,
}

impl ContextManager {
    pub async fn manage_context(&mut self, new_content: &str) -> Result<()> {
        let current_size = self.calculate_tokens();
        
        if current_size + new_content.len() > self.max_tokens {
            match self.preservation_strategy {
                PreservationStrategy::SlidingWindow => {
                    self.truncate_oldest();
                }
                PreservationStrategy::ImportanceBased { ref scorer } => {
                    self.remove_least_important(scorer).await?;
                }
                PreservationStrategy::Summarization { ref summarizer } => {
                    self.summarize_oldest(summarizer).await?;
                }
                PreservationStrategy::Adaptive => {
                    self.adaptive_compression().await?;
                }
            }
        }
        
        self.add_content(new_content);
        Ok(())
    }
}
```

## Specialized Implementations

### Code Generation with Dual Context

```rust
pub struct CodeGenerationDualContext {
    generator: CodeGenerator,
    evaluator: CodeEvaluator,
}

impl CodeGenerationDualContext {
    pub async fn generate_code(&self, spec: &CodeSpec) -> Result<Code> {
        // Generation phase
        let code = self.generator.generate(spec).await?;
        
        // Evaluation phase
        let evaluation = self.evaluator.evaluate(&code).await?;
        
        // Specific code evaluations
        let syntax_check = self.evaluator.check_syntax(&code)?;
        let type_check = self.evaluator.check_types(&code)?;
        let test_results = self.evaluator.run_tests(&code).await?;
        
        if !syntax_check.is_valid() || !type_check.is_valid() || !test_results.all_pass() {
            // Generate feedback
            let feedback = CodeFeedback {
                syntax_errors: syntax_check.errors,
                type_errors: type_check.errors,
                test_failures: test_results.failures,
            };
            
            // Retry with feedback
            return self.generator.regenerate_with_feedback(&code, feedback).await;
        }
        
        Ok(code)
    }
}
```

### Research Synthesis with Dual Context

```rust
pub struct ResearchSynthesisDualContext {
    generator: ResearchGenerator,
    evaluator: ResearchEvaluator,
}

impl ResearchSynthesisDualContext {
    pub async fn synthesize(&self, sources: &[Source]) -> Result<Synthesis> {
        // Generation phase
        let synthesis = self.generator.synthesize(sources).await?;
        
        // Evaluation phase
        let evaluation = self.evaluator.evaluate(&synthesis, sources).await?;
        
        // Check for hallucinations
        let fact_check = self.evaluator.verify_facts(&synthesis, sources)?;
        
        // Check for completeness
        let coverage = self.evaluator.check_coverage(&synthesis, sources)?;
        
        // Check for bias
        let bias_check = self.evaluator.check_bias(&synthesis)?;
        
        if !fact_check.all_verified() || coverage.percentage < 0.8 || bias_check.has_bias() {
            let feedback = ResearchFeedback {
                unverified_claims: fact_check.unverified,
                missing_topics: coverage.missing,
                bias_indicators: bias_check.indicators,
            };
            
            return self.generator.refine_synthesis(&synthesis, feedback).await;
        }
        
        Ok(synthesis)
    }
}
```

## Performance Metrics

### Quality Improvement Measurement

```rust
pub struct QualityMetrics {
    pub baseline_score: f64,
    pub dual_context_score: f64,
    pub improvement_percentage: f64,
    pub iteration_count: usize,
    pub evaluation_time: Duration,
}

impl QualityMetrics {
    pub fn calculate_improvement(&self) -> f64 {
        ((self.dual_context_score - self.baseline_score) / self.baseline_score) * 100.0
    }
    
    pub fn cost_benefit_ratio(&self) -> f64 {
        self.improvement_percentage / (self.iteration_count as f64)
    }
}
```

### A/B Testing Framework

```rust
pub struct DualContextABTest {
    pub single_context_agent: Box<dyn Agent>,
    pub dual_context_agent: DualContextAgent,
    pub test_cases: Vec<TestCase>,
}

impl DualContextABTest {
    pub async fn run_comparison(&self) -> ComparisonResults {
        let mut results = ComparisonResults::new();
        
        for test_case in &self.test_cases {
            let single_result = self.single_context_agent.execute(&test_case.task).await;
            let dual_result = self.dual_context_agent.execute(&test_case.task).await;
            
            results.add_comparison(test_case, single_result, dual_result);
        }
        
        results
    }
}
```

## Integration with Other Patterns

### With Resumable Workflows
- Checkpoint between generation and evaluation phases
- Resume can re-evaluate or accept previous evaluation
- Store evaluation history in checkpoint

### With Agent Reasoning Paradigms
- Use different paradigms for generation vs evaluation
- E.g., ToT for generation, CoT for evaluation
- Paradigm-specific evaluation criteria

### With Human-in-the-Loop
- Human can override evaluation scores
- Human feedback integrated into next iteration
- Evaluation criteria can be human-defined

## Best Practices

### Context Design
1. Keep contexts truly independent - no shared state
2. Use different prompting strategies for each context
3. Consider different models for each context
4. Reset contexts between iterations to avoid contamination

### Evaluation Criteria
1. Make criteria explicit and measurable
2. Use multiple orthogonal criteria
3. Weight criteria based on task requirements
4. Include both objective and subjective measures

### Iteration Management
1. Set reasonable iteration limits
2. Implement early stopping to save resources
3. Track improvement trends
4. Store best results even if not final

### Feedback Quality
1. Make feedback specific and actionable
2. Prioritize most impactful improvements
3. Avoid overwhelming generator with too much feedback
4. Structure feedback for easy integration

## Common Pitfalls and Solutions

### Pitfall: Evaluation Context Becomes Too Harsh
**Solution**: Balance criticism with recognition of strengths

### Pitfall: Infinite Refinement Loop
**Solution**: Implement convergence detection and early stopping

### Pitfall: Context Window Overflow
**Solution**: Implement smart context management and summarization

### Pitfall: Evaluation Criteria Mismatch
**Solution**: Align evaluation criteria with actual task requirements

## Conclusion

The dual-context evaluation pattern represents a powerful approach to improving agent output quality. By separating generation and evaluation into distinct contexts, we eliminate biases, catch errors, and enable iterative refinement that consistently produces better results than single-context approaches. The 40-60% quality improvement demonstrated in practice makes this pattern essential for production agent systems where output quality is critical.