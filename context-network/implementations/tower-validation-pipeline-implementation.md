# Tower Validation Pipeline Implementation

## Overview

Implementation guide for building a comprehensive validation pipeline using the Tower middleware ecosystem in Rust.

## Architecture

### Core Components

```rust
// Pipeline structure
pub struct ValidationPipeline<S> {
    inner: S,
    validators: Vec<Box<dyn Validator>>,
    config: ValidationConfig,
}

// Validator trait
#[async_trait]
pub trait Validator: Send + Sync {
    async fn validate(&self, request: &Request) -> Result<(), ValidationError>;
    fn name(&self) -> &'static str;
    fn priority(&self) -> u8; // Lower numbers = higher priority
}
```

### Layer Implementation

```rust
impl<S> tower::Layer<S> for ValidationLayer {
    type Service = ValidationPipeline<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ValidationPipeline {
            inner,
            validators: self.validators.clone(),
            config: self.config.clone(),
        }
    }
}

impl<S, B> tower::Service<Request<B>> for ValidationPipeline<S>
where
    S: tower::Service<Request<B>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let validators = self.validators.clone();
        let inner = self.inner.call(req);

        Box::pin(async move {
            // Run validation pipeline
            for validator in validators.iter() {
                validator.validate(&req).await?;
            }

            // Continue to next middleware
            inner.await
        })
    }
}
```

## Built-in Validators

### 1. Input Sanitization Validator

```rust
pub struct InputSanitizationValidator {
    html_policy: ammonia::Builder,
    max_input_size: usize,
}

#[async_trait]
impl Validator for InputSanitizationValidator {
    async fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        let body = extract_body(request).await?;

        // Size validation
        if body.len() > self.max_input_size {
            return Err(ValidationError::InputTooLarge {
                size: body.len(),
                max_size: self.max_input_size,
            });
        }

        // HTML sanitization
        let sanitized = self.html_policy.clean(&body);
        if sanitized != body {
            return Err(ValidationError::UnsafeHtmlDetected);
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "input_sanitization"
    }

    fn priority(&self) -> u8 {
        10 // High priority - run early
    }
}
```

### 2. Rate Limiting Validator

```rust
pub struct RateLimitValidator {
    limiter: Arc<governor::RateLimiter<String, governor::state::direct::NotKeyed>>,
    key_extractor: Box<dyn Fn(&Request) -> String + Send + Sync>,
}

#[async_trait]
impl Validator for RateLimitValidator {
    async fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        let key = (self.key_extractor)(request);

        match self.limiter.check_key(&key) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::RateLimitExceeded { key }),
        }
    }

    fn name(&self) -> &'static str {
        "rate_limiting"
    }

    fn priority(&self) -> u8 {
        5 // Very high priority - protect against abuse
    }
}
```

### 3. Schema Validation Validator

```rust
pub struct SchemaValidator {
    schemas: HashMap<String, serde_json::Value>,
    strict_mode: bool,
}

#[async_trait]
impl Validator for SchemaValidator {
    async fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        let content_type = request
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/json");

        if content_type.contains("application/json") {
            let body = extract_json_body(request).await?;
            let endpoint = extract_endpoint(request);

            if let Some(schema) = self.schemas.get(endpoint) {
                validate_json_schema(&body, schema, self.strict_mode)?;
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "schema_validation"
    }

    fn priority(&self) -> u8 {
        20 // After sanitization, before business logic
    }
}
```

### 4. Authentication Validator

```rust
pub struct AuthenticationValidator {
    jwt_decoder: jsonwebtoken::DecodingKey,
    required_scopes: Vec<String>,
}

#[async_trait]
impl Validator for AuthenticationValidator {
    async fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(ValidationError::MissingAuthentication)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ValidationError::InvalidAuthenticationFormat);
        }

        let token = &auth_header[7..];
        let claims: Claims = jsonwebtoken::decode(
            token,
            &self.jwt_decoder,
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| ValidationError::InvalidToken)?
        .claims;

        // Validate required scopes
        for required_scope in &self.required_scopes {
            if !claims.scopes.contains(required_scope) {
                return Err(ValidationError::InsufficientPermissions {
                    required: required_scope.clone(),
                });
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "authentication"
    }

    fn priority(&self) -> u8 {
        15 // After basic validation, before business logic
    }
}
```

## Configuration and Setup

### Pipeline Configuration

```rust
#[derive(Clone)]
pub struct ValidationConfig {
    pub enabled_validators: Vec<String>,
    pub fail_fast: bool,
    pub log_validation_failures: bool,
    pub metrics_enabled: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled_validators: vec![
                "rate_limiting".to_string(),
                "input_sanitization".to_string(),
                "authentication".to_string(),
                "schema_validation".to_string(),
            ],
            fail_fast: true,
            log_validation_failures: true,
            metrics_enabled: true,
        }
    }
}
```

### Builder Pattern

```rust
pub struct ValidationPipelineBuilder {
    validators: Vec<Box<dyn Validator>>,
    config: ValidationConfig,
}

impl ValidationPipelineBuilder {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            config: ValidationConfig::default(),
        }
    }

    pub fn with_rate_limiting(mut self, limiter: RateLimitValidator) -> Self {
        self.validators.push(Box::new(limiter));
        self
    }

    pub fn with_input_sanitization(mut self, sanitizer: InputSanitizationValidator) -> Self {
        self.validators.push(Box::new(sanitizer));
        self
    }

    pub fn with_schema_validation(mut self, validator: SchemaValidator) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    pub fn with_authentication(mut self, auth: AuthenticationValidator) -> Self {
        self.validators.push(Box::new(auth));
        self
    }

    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn build<S>(self) -> ValidationLayer {
        // Sort validators by priority
        let mut validators = self.validators;
        validators.sort_by_key(|v| v.priority());

        ValidationLayer {
            validators,
            config: self.config,
        }
    }
}
```

## Error Handling

### Validation Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Input size {size} exceeds maximum {max_size}")]
    InputTooLarge { size: usize, max_size: usize },

    #[error("Unsafe HTML content detected")]
    UnsafeHtmlDetected,

    #[error("Rate limit exceeded for key: {key}")]
    RateLimitExceeded { key: String },

    #[error("Schema validation failed: {details}")]
    SchemaValidationFailed { details: String },

    #[error("Missing authentication header")]
    MissingAuthentication,

    #[error("Invalid authentication format")]
    InvalidAuthenticationFormat,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Insufficient permissions: required {required}")]
    InsufficientPermissions { required: String },

    #[error("Internal validation error: {source}")]
    Internal { source: Box<dyn std::error::Error + Send + Sync> },
}
```

### Error Response Mapping

```rust
impl From<ValidationError> for Response<Body> {
    fn from(error: ValidationError) -> Self {
        let (status, message) = match error {
            ValidationError::InputTooLarge { .. } => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request payload too large",
            ),
            ValidationError::UnsafeHtmlDetected => (
                StatusCode::BAD_REQUEST,
                "Unsafe content detected",
            ),
            ValidationError::RateLimitExceeded { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded",
            ),
            ValidationError::SchemaValidationFailed { .. } => (
                StatusCode::BAD_REQUEST,
                "Invalid request format",
            ),
            ValidationError::MissingAuthentication |
            ValidationError::InvalidAuthenticationFormat |
            ValidationError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Authentication required",
            ),
            ValidationError::InsufficientPermissions { .. } => (
                StatusCode::FORBIDDEN,
                "Insufficient permissions",
            ),
            ValidationError::Internal { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
        };

        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({
                "error": message,
                "details": error.to_string()
            }).to_string()))
            .unwrap()
    }
}
```

## Testing Strategy

### Unit Tests for Individual Validators

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_input_sanitization_validator() {
        let validator = InputSanitizationValidator::new(1024);

        // Test valid input
        let clean_request = create_test_request("clean content");
        assert!(validator.validate(&clean_request).await.is_ok());

        // Test XSS attempt
        let xss_request = create_test_request("<script>alert('xss')</script>");
        assert!(validator.validate(&xss_request).await.is_err());

        // Test oversized input
        let large_request = create_test_request(&"x".repeat(2048));
        assert!(validator.validate(&large_request).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_validator() {
        let validator = create_rate_limit_validator(2, Duration::from_secs(60));

        let request = create_test_request("test");

        // First two requests should succeed
        assert!(validator.validate(&request).await.is_ok());
        assert!(validator.validate(&request).await.is_ok());

        // Third request should fail
        assert!(validator.validate(&request).await.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_validation_pipeline() {
    let pipeline = ValidationPipelineBuilder::new()
        .with_rate_limiting(create_rate_limiter())
        .with_input_sanitization(create_sanitizer())
        .with_authentication(create_auth_validator())
        .build();

    let app = ServiceBuilder::new()
        .layer(pipeline)
        .service(echo_service());

    // Test valid authenticated request
    let valid_request = create_authenticated_request("valid content");
    let response = app.oneshot(valid_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test invalid request
    let invalid_request = create_request_with_xss();
    let response = app.oneshot(invalid_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

## Performance Considerations

### Optimization Strategies
- **Validator Ordering**: Run fastest/cheapest validators first
- **Early Exit**: Fail fast on first validation error
- **Caching**: Cache validation results where appropriate
- **Async Processing**: Non-blocking validation operations

### Metrics and Monitoring
- Validation latency per validator
- Validation failure rates by type
- Request throughput impact
- Resource usage monitoring

## Related Documentation

- [Validation TDD Methodology](../methodologies/validation-tdd-methodology.md)
- [HTML Sanitization Upgrade](html-sanitization-upgrade.md)
- [Validator Sorting Optimization](validator-sorting-optimization.md)

## Future Enhancements

- Plugin architecture for custom validators
- Dynamic validator configuration
- Validation result caching
- Integration with OpenAPI schema validation
- Support for async validator dependencies