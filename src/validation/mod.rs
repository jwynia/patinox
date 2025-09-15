//! Validation module for the Patinox framework
//!
//! This module provides Tower middleware integration for validation layers
//! and implements specific validators for security and quality assurance.

pub mod validators;

use crate::error::PatinoxError;
use crate::traits::validator::{ValidationRequest, ValidationResponse, Validator};
use std::sync::Arc;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

/// Tower layer that adds validation middleware to a service
#[derive(Clone)]
pub struct ValidationLayer {
    validators: Vec<Arc<dyn Validator>>,
}

impl ValidationLayer {
    /// Create a new validation layer with the specified validators
    pub fn new(validators: Vec<Arc<dyn Validator>>) -> Self {
        Self { validators }
    }

    /// Create a validation layer with default validators
    pub fn with_default_validators(llm_provider: Arc<dyn crate::provider::ModelProvider>) -> Self {
        let validators: Vec<Arc<dyn Validator>> = vec![
            Arc::new(validators::RequestValidator::new(
                validators::RequestValidatorConfig::default()
            )),
            Arc::new(validators::AntiJailbreakValidator::new(
                llm_provider.clone(),
                validators::AntiJailbreakConfig::default()
            )),
            Arc::new(validators::HallucinationDetector::new(
                llm_provider,
                validators::HallucinationConfig::default()
            )),
        ];
        Self::new(validators)
    }
}

impl<S> Layer<S> for ValidationLayer {
    type Service = ValidationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ValidationService {
            inner,
            validators: self.validators.clone(),
        }
    }
}

/// Tower service that performs validation before calling the inner service
#[derive(Clone)]
pub struct ValidationService<S> {
    inner: S,
    validators: Vec<Arc<dyn Validator>>,
}

impl<S> Service<ValidationRequest> for ValidationService<S>
where
    S: Service<ValidationRequest, Response = ValidationResponse, Error = PatinoxError>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    type Response = ValidationResponse;
    type Error = PatinoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ValidationRequest) -> Self::Future {
        let mut inner = self.inner.clone();
        let validators = self.validators.clone();

        Box::pin(async move {
            // Sort validators by priority (lower numbers first)
            let mut sorted_validators = validators;
            sorted_validators.sort_by_key(|v| v.config().priority);

            // Run validators in priority order
            for validator in sorted_validators {
                if validator.should_validate(&req) {
                    let validation_result = validator.validate(req.clone()).await?;

                    if !validation_result.approved {
                        // Return the rejection response
                        return Ok(validation_result);
                    }
                }
            }

            // If all validators pass, call the inner service
            inner.call(req).await
        })
    }
}

/// Builder for creating validation pipelines
pub struct ValidationPipeline {
    validators: Vec<Arc<dyn Validator>>,
}

impl ValidationPipeline {
    /// Create a new pipeline builder
    pub fn builder() -> ValidationPipelineBuilder {
        ValidationPipelineBuilder {
            validators: Vec::new(),
        }
    }

    /// Validate a request using all validators in the pipeline
    pub async fn validate(&self, request: ValidationRequest) -> Result<ValidationResponse, PatinoxError> {
        // Sort validators by priority
        let mut sorted_validators = self.validators.clone();
        sorted_validators.sort_by_key(|v| v.config().priority);

        for validator in sorted_validators {
            if validator.should_validate(&request) {
                let result = validator.validate(request.clone()).await?;

                if !result.approved {
                    return Ok(result);
                }
            }
        }

        // If all validators pass, approve the request
        Ok(ValidationResponse {
            approved: true,
            reason: Some("All validators passed".to_string()),
            modifications: None,
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Builder for constructing validation pipelines
pub struct ValidationPipelineBuilder {
    validators: Vec<Arc<dyn Validator>>,
}

impl ValidationPipelineBuilder {
    /// Add a request validator to the pipeline
    pub fn add_request_validator(mut self, config: validators::RequestValidatorConfig) -> Self {
        self.validators.push(Arc::new(validators::RequestValidator::new(config)));
        self
    }

    /// Add an anti-jailbreak validator to the pipeline
    pub fn add_anti_jailbreak_validator(
        mut self,
        llm_provider: Arc<dyn crate::provider::ModelProvider>,
        config: validators::AntiJailbreakConfig,
    ) -> Self {
        self.validators.push(Arc::new(validators::AntiJailbreakValidator::new(
            llm_provider, config
        )));
        self
    }

    /// Add a hallucination detector to the pipeline
    pub fn add_hallucination_detector(
        mut self,
        llm_provider: Arc<dyn crate::provider::ModelProvider>,
        config: validators::HallucinationConfig,
    ) -> Self {
        self.validators.push(Arc::new(validators::HallucinationDetector::new(
            llm_provider, config
        )));
        self
    }

    /// Add a custom validator to the pipeline
    pub fn add_custom_validator(mut self, validator: Arc<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Build the validation pipeline
    pub fn build(self) -> ValidationPipeline {
        ValidationPipeline {
            validators: self.validators,
        }
    }
}