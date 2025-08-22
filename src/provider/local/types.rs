//! Type definitions for local model providers

use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Local service type preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalService {
    Ollama,
    LMStudio,
    Auto, // Auto-detect best service
}

/// Performance metrics for a service
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    /// Average response time for health checks
    pub avg_response_time: Duration,
    
    /// Number of models available
    pub model_count: usize,
    
    /// Last request timestamp
    pub last_request: Option<Instant>,
    
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time: Duration::from_millis(100),
            model_count: 0,
            last_request: None,
            success_rate: 1.0,
        }
    }
}