//! Monitor trait definition and supporting types
//!
//! This module defines the core Monitor trait for asynchronous monitoring
//! of agent behavior and performance. Monitors collect telemetry data
//! for analysis and improvement of agent systems.

use crate::error::PatinoxError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Tests written FIRST to define the contract
#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct TestMonitor {
        name: String,
        config: MonitorConfig,
        events: std::sync::Arc<std::sync::Mutex<Vec<MonitorEvent>>>,
    }

    #[async_trait]
    impl Monitor for TestMonitor {
        fn name(&self) -> &str {
            &self.name
        }

        async fn start_monitoring(&self, execution_id: Uuid, agent_id: Uuid) -> Result<(), PatinoxError> {
            // Start monitoring by recording a start event
            let event = MonitorEvent {
                id: Uuid::new_v4(),
                execution_id,
                agent_id,
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::ExecutionStarted,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            };
            
            let mut events = self.events.lock().map_err(|_| {
                PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                    "Monitor event storage corrupted".to_string()
                ))
            })?;
            events.push(event);
            Ok(())
        }

        async fn record_event(&self, event: MonitorEvent) -> Result<(), PatinoxError> {
            let mut events = self.events.lock().map_err(|_| {
                PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                    "Monitor event storage corrupted".to_string()
                ))
            })?;
            events.push(event);
            Ok(())
        }

        async fn complete_monitoring(&self, execution_id: Uuid, summary: ExecutionSummary) -> Result<(), PatinoxError> {
            let event = MonitorEvent {
                id: Uuid::new_v4(),
                execution_id,
                agent_id: summary.agent_id,
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::ExecutionCompleted {
                    success: summary.success,
                    total_duration_ms: summary.total_duration_ms,
                },
                data: serde_json::to_value(&summary).unwrap(),
                metadata: HashMap::new(),
            };
            
            let mut events = self.events.lock().map_err(|_| {
                PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                    "Monitor event storage corrupted".to_string()
                ))
            })?;
            events.push(event);
            Ok(())
        }

        async fn query_events(&self, query: MonitorQuery) -> Result<Vec<MonitorEvent>, PatinoxError> {
            let events = self.events.lock().map_err(|_| {
                PatinoxError::Execution(crate::error::ExecutionError::ResourceExhausted(
                    "Monitor event storage corrupted".to_string()
                ))
            })?;
            let mut filtered_events = Vec::new();

            for event in events.iter() {
                let mut matches = true;

                // Filter by agent IDs
                if let Some(ref agent_ids) = query.agent_ids {
                    if !agent_ids.contains(&event.agent_id) {
                        matches = false;
                    }
                }

                // Filter by event types
                if let Some(ref event_types) = query.event_types {
                    if !event_types.iter().any(|et| std::mem::discriminant(et) == std::mem::discriminant(&event.event_type)) {
                        matches = false;
                    }
                }

                // Filter by time range
                if let Some(start_time) = query.start_time {
                    if event.timestamp < start_time {
                        matches = false;
                    }
                }

                if let Some(end_time) = query.end_time {
                    if event.timestamp > end_time {
                        matches = false;
                    }
                }

                if matches {
                    filtered_events.push(event.clone());
                }

                // Apply limit
                if let Some(limit) = query.limit {
                    if filtered_events.len() >= limit as usize {
                        break;
                    }
                }
            }

            Ok(filtered_events)
        }

        fn config(&self) -> &MonitorConfig {
            &self.config
        }
    }

    impl TestMonitor {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                config: MonitorConfig {
                    name: name.to_string(),
                    enabled: true,
                    buffer_size: 1000,
                    flush_interval_ms: 5000,
                    sampling_rate: 1.0,
                    event_types: vec![
                        MonitorEventType::ExecutionStarted,
                        MonitorEventType::ExecutionCompleted { success: true, total_duration_ms: 0 },
                    ],
                },
                events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn event_count(&self) -> usize {
            self.events.lock().map(|events| events.len()).unwrap_or(0)
        }
    }

    #[test]
    fn monitor_event_serialization() {
        let event = MonitorEvent {
            id: Uuid::new_v4(),
            execution_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event_type: MonitorEventType::LlmCalled {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                tokens: crate::traits::Usage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    total_tokens: 150,
                    cost_usd: Some(0.003),
                },
            },
            data: serde_json::json!({"additional": "data"}),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("request_id".to_string(), "req-123".to_string());
                meta
            },
        };

        let serialized = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: MonitorEvent = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.execution_id, event.execution_id);
        assert_eq!(deserialized.agent_id, event.agent_id);
        assert_eq!(deserialized.data["additional"], "data");
        assert_eq!(deserialized.metadata["request_id"], "req-123");

        // Test event type specifics
        match deserialized.event_type {
            MonitorEventType::LlmCalled { provider, model, tokens } => {
                assert_eq!(provider, "openai");
                assert_eq!(model, "gpt-4");
                assert_eq!(tokens.total_tokens, 150);
            },
            _ => panic!("Expected LlmCalled event type"),
        }
    }

    #[test]
    fn monitor_event_type_variants() {
        let event_types = vec![
            MonitorEventType::ExecutionStarted,
            MonitorEventType::ValidationPassed { validator: "test".to_string() },
            MonitorEventType::ValidationFailed { validator: "test".to_string(), reason: "failed".to_string() },
            MonitorEventType::ToolExecuted { tool: "calc".to_string(), duration_ms: 150 },
            MonitorEventType::LlmCalled {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                tokens: crate::traits::Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    cost_usd: None,
                },
            },
            MonitorEventType::ErrorOccurred { error_type: "timeout".to_string(), recoverable: true },
            MonitorEventType::ExecutionCompleted { success: true, total_duration_ms: 2500 },
        ];

        for event_type in event_types {
            // All variants should be debuggable, cloneable, and serializable
            let _debug = format!("{:?}", event_type);
            let _cloned = event_type.clone();
            let _serialized = serde_json::to_string(&event_type).expect("Should serialize");
        }
    }

    #[test]
    fn execution_summary_serialization() {
        let summary = ExecutionSummary {
            execution_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            success: true,
            total_duration_ms: 5000,
            llm_calls: 3,
            tool_calls: 5,
            validation_failures: 1,
            total_tokens: crate::traits::Usage {
                prompt_tokens: 300,
                completion_tokens: 150,
                total_tokens: 450,
                cost_usd: Some(0.009),
            },
            error_summary: Some("Minor validation warning".to_string()),
        };

        let serialized = serde_json::to_string(&summary).expect("Should serialize");
        let deserialized: ExecutionSummary = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.execution_id, summary.execution_id);
        assert_eq!(deserialized.agent_id, summary.agent_id);
        assert_eq!(deserialized.success, summary.success);
        assert_eq!(deserialized.total_duration_ms, summary.total_duration_ms);
        assert_eq!(deserialized.llm_calls, summary.llm_calls);
        assert_eq!(deserialized.tool_calls, summary.tool_calls);
        assert_eq!(deserialized.validation_failures, summary.validation_failures);
        assert_eq!(deserialized.total_tokens.total_tokens, summary.total_tokens.total_tokens);
        assert_eq!(deserialized.error_summary, summary.error_summary);
    }

    #[test]
    fn monitor_query_structure() {
        let query = MonitorQuery {
            agent_ids: Some(vec![Uuid::new_v4(), Uuid::new_v4()]),
            event_types: Some(vec![
                MonitorEventType::ExecutionStarted,
                MonitorEventType::ExecutionCompleted { success: true, total_duration_ms: 0 },
            ]),
            start_time: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            end_time: Some(chrono::Utc::now()),
            limit: Some(100),
        };

        // Should be debuggable and cloneable
        let _debug = format!("{:?}", query);
        let cloned = query.clone();
        assert_eq!(cloned.agent_ids.as_ref().expect("Should have agent IDs").len(), 2);
        assert_eq!(cloned.event_types.as_ref().expect("Should have event types").len(), 2);
        assert_eq!(cloned.limit, Some(100));
    }

    #[test]
    fn monitor_config_serialization() {
        let config = MonitorConfig {
            name: "test-monitor".to_string(),
            enabled: true,
            buffer_size: 5000,
            flush_interval_ms: 10000,
            sampling_rate: 0.8,
            event_types: vec![
                MonitorEventType::ExecutionStarted,
                MonitorEventType::ValidationFailed { validator: "test".to_string(), reason: "test".to_string() },
            ],
        };

        let serialized = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: MonitorConfig = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.name, config.name);
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.buffer_size, config.buffer_size);
        assert_eq!(deserialized.flush_interval_ms, config.flush_interval_ms);
        assert_eq!(deserialized.sampling_rate, config.sampling_rate);
        assert_eq!(deserialized.event_types.len(), config.event_types.len());
    }

    #[tokio::test]
    async fn monitor_basic_functionality() {
        let monitor = TestMonitor::new("test-monitor");

        // Test metadata access
        assert_eq!(monitor.name(), "test-monitor");
        
        let config = monitor.config();
        assert_eq!(config.name, "test-monitor");
        assert!(config.enabled);
        assert!(config.buffer_size > 0);
        assert!(config.flush_interval_ms > 0);
        assert!(config.sampling_rate > 0.0 && config.sampling_rate <= 1.0);
    }

    #[tokio::test]
    async fn monitor_execution_lifecycle() {
        let monitor = TestMonitor::new("lifecycle-test");
        let execution_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        assert_eq!(monitor.event_count(), 0);

        // Start monitoring
        let result = monitor.start_monitoring(execution_id, agent_id).await;
        assert!(result.is_ok());
        assert_eq!(monitor.event_count(), 1);

        // Record some events
        let event1 = MonitorEvent {
            id: Uuid::new_v4(),
            execution_id,
            agent_id,
            timestamp: chrono::Utc::now(),
            event_type: MonitorEventType::ValidationPassed { validator: "test".to_string() },
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };

        let result = monitor.record_event(event1).await;
        assert!(result.is_ok());
        assert_eq!(monitor.event_count(), 2);

        let event2 = MonitorEvent {
            id: Uuid::new_v4(),
            execution_id,
            agent_id,
            timestamp: chrono::Utc::now(),
            event_type: MonitorEventType::ToolExecuted { tool: "calculator".to_string(), duration_ms: 250 },
            data: serde_json::json!({"result": "42"}),
            metadata: HashMap::new(),
        };

        let result = monitor.record_event(event2).await;
        assert!(result.is_ok());
        assert_eq!(monitor.event_count(), 3);

        // Complete monitoring
        let summary = ExecutionSummary {
            execution_id,
            agent_id,
            success: true,
            total_duration_ms: 1500,
            llm_calls: 1,
            tool_calls: 2,
            validation_failures: 0,
            total_tokens: crate::traits::Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                cost_usd: Some(0.003),
            },
            error_summary: None,
        };

        let result = monitor.complete_monitoring(execution_id, summary).await;
        assert!(result.is_ok());
        assert_eq!(monitor.event_count(), 4);
    }

    #[tokio::test]
    async fn monitor_query_functionality() {
        let monitor = TestMonitor::new("query-test");
        let agent1_id = Uuid::new_v4();
        let agent2_id = Uuid::new_v4();
        let execution1_id = Uuid::new_v4();
        let execution2_id = Uuid::new_v4();

        // Add some test events
        let events = vec![
            MonitorEvent {
                id: Uuid::new_v4(),
                execution_id: execution1_id,
                agent_id: agent1_id,
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(10),
                event_type: MonitorEventType::ExecutionStarted,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            MonitorEvent {
                id: Uuid::new_v4(),
                execution_id: execution2_id,
                agent_id: agent2_id,
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(5),
                event_type: MonitorEventType::ValidationPassed { validator: "test".to_string() },
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            MonitorEvent {
                id: Uuid::new_v4(),
                execution_id: execution1_id,
                agent_id: agent1_id,
                timestamp: chrono::Utc::now(),
                event_type: MonitorEventType::ExecutionCompleted { success: true, total_duration_ms: 600000 },
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
        ];

        for event in events {
            monitor.record_event(event).await.unwrap();
        }

        // Query all events
        let query = MonitorQuery {
            agent_ids: None,
            event_types: None,
            start_time: None,
            end_time: None,
            limit: None,
        };

        let result = monitor.query_events(query).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("All events query should succeed").len(), 3);

        // Query by agent ID
        let query = MonitorQuery {
            agent_ids: Some(vec![agent1_id]),
            event_types: None,
            start_time: None,
            end_time: None,
            limit: None,
        };

        let result = monitor.query_events(query).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Agent ID query should succeed").len(), 2);

        // Query by event type
        let query = MonitorQuery {
            agent_ids: None,
            event_types: Some(vec![MonitorEventType::ExecutionStarted]),
            start_time: None,
            end_time: None,
            limit: None,
        };

        let result = monitor.query_events(query).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Event type query should succeed").len(), 1);

        // Query with limit
        let query = MonitorQuery {
            agent_ids: None,
            event_types: None,
            start_time: None,
            end_time: None,
            limit: Some(1),
        };

        let result = monitor.query_events(query).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("Limit query should succeed").len(), 1);
    }

    #[tokio::test]
    async fn monitor_object_safety() {
        // Test that we can create trait objects
        let monitor: Box<dyn Monitor> = Box::new(TestMonitor::new("boxed-monitor"));

        // Test that trait object methods work
        let _name = monitor.name();
        let _config = monitor.config();

        let execution_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        // Test async methods work with trait objects
        let _result = monitor.start_monitoring(execution_id, agent_id).await;

        let event = MonitorEvent {
            id: Uuid::new_v4(),
            execution_id,
            agent_id,
            timestamp: chrono::Utc::now(),
            event_type: MonitorEventType::ExecutionStarted,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };

        let _result = monitor.record_event(event).await;

        // Test that we can store multiple monitors in a collection
        let monitors: Vec<Box<dyn Monitor>> = vec![
            Box::new(TestMonitor::new("monitor1")),
            Box::new(TestMonitor::new("monitor2")),
        ];

        assert_eq!(monitors.len(), 2);
    }

    #[tokio::test]
    async fn monitor_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Box<dyn Monitor>>();
        assert_sync::<Box<dyn Monitor>>();

        // Test that we can pass trait objects across thread boundaries
        let monitor: Box<dyn Monitor> = Box::new(TestMonitor::new("thread-test"));
        let monitor_name = monitor.name().to_string();

        tokio::spawn(async move {
            let _name = monitor.name();
            // Monitor trait object can be moved across threads
        }).await.unwrap();

        assert_eq!(monitor_name, "thread-test");
    }

    #[test]
    fn usage_type_functionality() {
        let usage = crate::traits::Usage {
            prompt_tokens: 150,
            completion_tokens: 75,
            total_tokens: 225,
            cost_usd: Some(0.0045),
        };

        // Should be debuggable and cloneable
        let _debug = format!("{:?}", usage);
        let cloned = usage.clone();
        assert_eq!(cloned.prompt_tokens, usage.prompt_tokens);
        assert_eq!(cloned.completion_tokens, usage.completion_tokens);
        assert_eq!(cloned.total_tokens, usage.total_tokens);
        assert_eq!(cloned.cost_usd, usage.cost_usd);

        // Should be serializable
        let serialized = serde_json::to_string(&usage).expect("Should serialize");
        let deserialized: crate::traits::Usage = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.total_tokens, usage.total_tokens);
    }
}

/// Asynchronous monitoring for agent behavior analysis
#[async_trait]
pub trait Monitor: Send + Sync {
    /// Monitor identifier
    fn name(&self) -> &str;
    
    /// Start monitoring an agent execution
    async fn start_monitoring(&self, execution_id: Uuid, agent_id: Uuid) -> Result<(), PatinoxError>;
    
    /// Record an event during execution
    async fn record_event(&self, event: MonitorEvent) -> Result<(), PatinoxError>;
    
    /// Complete monitoring for an execution
    async fn complete_monitoring(&self, execution_id: Uuid, summary: ExecutionSummary) -> Result<(), PatinoxError>;
    
    /// Query monitoring data for analysis
    async fn query_events(&self, query: MonitorQuery) -> Result<Vec<MonitorEvent>, PatinoxError>;
    
    /// Get monitoring configuration
    fn config(&self) -> &MonitorConfig;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEvent {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: MonitorEventType,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorEventType {
    ExecutionStarted,
    ValidationPassed { validator: String },
    ValidationFailed { validator: String, reason: String },
    ToolExecuted { tool: String, duration_ms: u64 },
    LlmCalled { provider: String, model: String, tokens: crate::traits::Usage },
    ErrorOccurred { error_type: String, recoverable: bool },
    ExecutionCompleted { success: bool, total_duration_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub success: bool,
    pub total_duration_ms: u64,
    pub llm_calls: u32,
    pub tool_calls: u32,
    pub validation_failures: u32,
    pub total_tokens: crate::traits::Usage,
    pub error_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorQuery {
    pub agent_ids: Option<Vec<Uuid>>,
    pub event_types: Option<Vec<MonitorEventType>>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub name: String,
    pub enabled: bool,
    pub buffer_size: u32,
    pub flush_interval_ms: u64,
    pub sampling_rate: f64, // 0.0 to 1.0
    pub event_types: Vec<MonitorEventType>,
}