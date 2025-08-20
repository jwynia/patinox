//! Examples demonstrating compile-time type safety
//!
//! These examples show how the typestate pattern prevents invalid operations
//! at compile time, catching errors before runtime.

use patinox::prelude::*;
use std::collections::HashMap;

fn main() {
    println!("Type Safety Infrastructure Examples");
    println!("==================================");

    // Example 1: Valid agent state transitions
    println!("\n1. Valid Agent State Transitions:");
    valid_state_transitions();

    // Example 2: Type-safe builder pattern
    println!("\n2. Type-Safe Builder Pattern:");
    type_safe_builder_example();

    // Example 3: Compile-time prevention of invalid operations
    println!("\n3. Compile-Time Safety (see comments for prevented operations):");
    compile_time_safety_examples();
}

fn valid_state_transitions() {
    // Create agent configuration
    let config = AgentConfig {
        name: "example-agent".to_string(),
        description: Some("Example agent for demonstration".to_string()),
        max_concurrent_requests: 10,
        timeout_ms: 30000,
        enabled_validators: vec!["safety-check".to_string()],
        llm_provider: "openai".to_string(),
        llm_model: "gpt-4".to_string(),
        tools: vec!["search".to_string(), "calculator".to_string()],
        metadata: HashMap::new(),
    };

    // Create agent in Created state
    let agent = AgentWrapper::<Created>::new(config);
    println!(
        "  ✓ Agent created in Created state: {}",
        agent.current_state()
    );

    // Transition: Created -> Started
    let agent = agent.start().unwrap();
    println!(
        "  ✓ Agent transitioned to Started state: {}",
        agent.current_state()
    );

    // Transition: Started -> Running
    let agent = agent.run().unwrap();
    println!(
        "  ✓ Agent transitioned to Running state: {}",
        agent.current_state()
    );

    // Only now can the agent execute requests
    println!("  ✓ Agent can execute: {}", agent.can_execute());

    // Transition: Running -> Stopped
    let agent = agent.stop().unwrap();
    println!(
        "  ✓ Agent transitioned to Stopped state: {}",
        agent.current_state()
    );
    println!("  ✓ Agent can execute after stop: {}", agent.can_execute());
}

fn type_safe_builder_example() {
    // Start with empty builder
    let builder = ConfigBuilder::<EmptyBuilder>::new("demo-agent");
    println!(
        "  ✓ Created builder in Empty state: {}",
        builder.builder_state()
    );
    println!(
        "  ✓ Missing required fields: {}",
        builder.missing_required_count()
    );

    // Add optional fields (stays in EmptyBuilder state)
    let builder = builder
        .description("Demo agent")
        .max_concurrent_requests(15)
        .timeout_ms(45000)
        .add_validator("input-sanitizer")
        .add_tool("web-search");

    println!(
        "  ✓ Added optional fields, still in: {}",
        builder.builder_state()
    );
    println!(
        "  ✓ Still missing required fields: {}",
        builder.missing_required_count()
    );

    // Add first required field (transitions to PartialBuilder)
    let builder = builder.llm_provider("openai");
    println!(
        "  ✓ Added LLM provider, now in: {}",
        builder.builder_state()
    );
    println!(
        "  ✓ Missing required fields: {}",
        builder.missing_required_count()
    );

    // Add second required field (transitions to CompleteBuilder)
    let builder = builder.llm_model("gpt-4");
    println!("  ✓ Added LLM model, now in: {}", builder.builder_state());
    println!(
        "  ✓ Missing required fields: {}",
        builder.missing_required_count()
    );

    // Only CompleteBuilder can build
    let config = builder.build();
    println!("  ✓ Successfully built configuration: {}", config.name);
}

fn compile_time_safety_examples() {
    let config = AgentConfig {
        name: "safety-demo".to_string(),
        description: Some("Demonstrates compile-time safety".to_string()),
        max_concurrent_requests: 5,
        timeout_ms: 30000,
        enabled_validators: vec![],
        llm_provider: "openai".to_string(),
        llm_model: "gpt-4".to_string(),
        tools: vec![],
        metadata: HashMap::new(),
    };

    let agent = AgentWrapper::<Created>::new(config);

    // These operations would FAIL TO COMPILE (uncomment to see errors):

    // ❌ Cannot execute in Created state
    // let request = AgentRequest { /* ... */ };
    // let response = agent.execute(request); // COMPILE ERROR

    // ❌ Cannot transition Created directly to Running
    // let running_agent = agent.run(); // COMPILE ERROR - method doesn't exist

    println!(
        "  ✓ Created state agent cannot execute: {}",
        agent.can_execute()
    );

    let agent = agent.start().unwrap();
    println!(
        "  ✓ Started state agent cannot execute: {}",
        agent.can_execute()
    );

    // ❌ Cannot execute in Started state either
    // let response = agent.execute(request); // COMPILE ERROR

    let agent = agent.run().unwrap();
    println!(
        "  ✓ Running state agent CAN execute: {}",
        agent.can_execute()
    );

    // ✅ Only Running state allows execution
    // let response = agent.execute(request); // This would compile

    println!("  ✓ Type system prevents invalid operations at compile time!");
}

// Demonstrate builder compile-time safety
#[allow(dead_code)]
fn builder_compile_time_examples() {
    // These would FAIL TO COMPILE (uncomment to see errors):

    // ❌ Cannot build EmptyBuilder
    // let config = ConfigBuilder::<EmptyBuilder>::new("test").build(); // COMPILE ERROR

    // ❌ Cannot build PartialBuilder
    // let config = ConfigBuilder::<EmptyBuilder>::new("test")
    //     .llm_provider("openai")
    //     .build(); // COMPILE ERROR

    // ✅ Only CompleteBuilder can build
    let _config = ConfigBuilder::<EmptyBuilder>::new("test")
        .llm_provider("openai")
        .llm_model("gpt-4")
        .build(); // ✓ This compiles

    println!("  ✓ Builder enforces required fields at compile time!");
}
