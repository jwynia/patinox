# Lifecycle Hook Use Cases Catalog

## Purpose
Comprehensive catalog of use cases for each of the 6 lifecycle hooks in Patinox V2 architecture. This document captures validated pain points from external agent framework experience to guide future hook implementation prioritization.

## Classification
- **Domain:** Architecture Reference
- **Stability:** Evolving (grows with experience)
- **Abstraction:** Conceptual
- **Confidence:** High (validated by production experience)

## Source of Use Cases

**Primary**: Project lead's production experience with other agent frameworks
**Secondary**: LangChain V1 middleware announcement (industry validation)
**Purpose**: Guide Layer 3+ decisions on which hooks to implement when

---

## Hook 1: `before_agent`

**Called**: Before agent starts processing any input
**Signature**: `async fn before_agent(&self, input: &str) -> Result<String>`
**Default**: Passthrough (returns input unchanged)

### Use Cases

#### UC-1.1: Input Sanitization
**Pain Point**: Raw user input contains PII, sensitive data, or malformed content
**Solution**: Strip PII, validate format, normalize encoding
**Priority**: High (security/privacy concern)
**Example**:
```rust
async fn before_agent(&self, input: &str) -> Result<String> {
    // Strip credit card numbers, SSNs, email addresses
    let sanitized = self.pii_filter.remove_sensitive_data(input)?;
    Ok(sanitized)
}
```

#### UC-1.2: Rate Limiting / Quota Checks
**Pain Point**: Need to enforce per-user request limits before expensive LLM calls
**Solution**: Check user quota, reject if exceeded, track usage
**Priority**: High (cost control)
**Example**:
```rust
async fn before_agent(&self, input: &str) -> Result<String> {
    if !self.quota_manager.check_user_quota(user_id).await? {
        return Err("Rate limit exceeded".into());
    }
    self.quota_manager.increment(user_id).await?;
    Ok(input.to_string())
}
```

#### UC-1.3: Context Loading
**Pain Point**: Agent needs relevant history, documents, or state before processing
**Solution**: Fetch conversation history, load relevant documents from vector DB
**Priority**: Medium (enhances quality)
**Example**:
```rust
async fn before_agent(&self, input: &str) -> Result<String> {
    let history = self.db.get_recent_history(session_id, 10).await?;
    let context = format!("Previous conversation:\n{}\n\nCurrent: {}", history, input);
    Ok(context)
}
```

#### UC-1.4: Request Routing
**Pain Point**: Need to select which specialized agent handles the request
**Solution**: Analyze input, route to appropriate agent based on intent
**Priority**: Low (Layer 3 multi-agent feature)
**Example**:
```rust
async fn before_agent(&self, input: &str) -> Result<String> {
    let intent = self.classifier.classify(input).await?;
    if intent.is_code_related() {
        return self.route_to_code_agent(input);
    }
    Ok(input.to_string())
}
```

#### UC-1.5: Input Validation
**Pain Point**: Invalid input causes agent to waste LLM calls or produce errors
**Solution**: Validate input meets expected schema, reject early if malformed
**Priority**: Medium (UX improvement)
**Example**:
```rust
async fn before_agent(&self, input: &str) -> Result<String> {
    if input.is_empty() || input.len() > MAX_INPUT_LENGTH {
        return Err("Invalid input length".into());
    }
    Ok(input.to_string())
}
```

### Implementation Priority

**Layer 2**: None (not validated as pain yet)
**Layer 3**: Input validation, rate limiting (if production pain)
**Layer 4**: PII sanitization, context loading (enterprise features)

---

## Hook 2: `before_model`

**Called**: Before sending messages to LLM provider
**Signature**: `async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>>`
**Default**: Passthrough (returns messages unchanged)

### Use Cases

#### UC-2.1: Context Window Management
**Pain Point**: Conversation history exceeds model's token limit, causes failures
**Solution**: Trim old messages, summarize context, compress history
**Priority**: High (will hit this with long conversations)
**Example**:
```rust
async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
    let token_count = self.count_tokens(&messages);
    if token_count > MAX_CONTEXT_TOKENS {
        // Summarize old messages, keep recent ones
        let summarized = self.summarize_history(&messages[..N]).await?;
        let recent = messages[N..].to_vec();
        return Ok(vec![summarized].into_iter().chain(recent).collect());
    }
    Ok(messages)
}
```

#### UC-2.2: Dynamic Prompt Injection
**Pain Point**: Need to add instructions based on current state/context
**Solution**: Inject system messages with dynamic content (time, user info, constraints)
**Priority**: Medium (useful for adaptive behavior)
**Example**:
```rust
async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
    let current_time = Utc::now().format("%Y-%m-%d %H:%M UTC");
    let constraint = Message::system(format!(
        "Current time: {}. Be concise, max 2 paragraphs.", current_time
    ));
    Ok(vec![constraint].into_iter().chain(messages).collect())
}
```

#### UC-2.3: Message Deduplication
**Pain Point**: Repeated messages waste tokens and confuse model
**Solution**: Remove duplicate consecutive messages, collapse redundant content
**Priority**: Low (edge case optimization)
**Example**:
```rust
async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
    let mut dedupe = Vec::new();
    let mut prev: Option<&Message> = None;
    for msg in &messages {
        if prev.map_or(true, |p| p.content != msg.content) {
            dedupe.push(msg.clone());
        }
        prev = Some(msg);
    }
    Ok(dedupe)
}
```

#### UC-2.4: Cost Optimization
**Pain Point**: Sending full messages expensive, need compression
**Solution**: Compress messages, use cheaper models for simple queries
**Priority**: Medium (cost control)
**Example**:
```rust
async fn before_model(&self, messages: Vec<Message>) -> Result<Vec<Message>> {
    let compressed = messages.iter().map(|m| {
        Message {
            role: m.role.clone(),
            content: self.compress_content(&m.content),
        }
    }).collect();
    Ok(compressed)
}
```

### Implementation Priority

**Layer 2**: None (haven't hit token limits yet)
**Layer 3**: Context window management (high priority - will hit this)
**Layer 4**: Dynamic prompts, cost optimization (enterprise)

---

## Hook 3: `wrap_model_call`

**Called**: Wraps every LLM provider call
**Signature**: `async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>`
**Default**: Direct call (`f().await`)

### Use Cases

#### UC-3.1: Retry with Exponential Backoff
**Pain Point**: Transient API failures (429, 503) cause agent to fail
**Solution**: Retry failed calls with backoff, up to N attempts
**Priority**: High (API reliability is critical)
**Example**:
```rust
async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse>
where F: Fn() -> Fut, Fut: Future<Output = ProviderResult<ProviderResponse>>
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(response) => return Ok(response),
            Err(e) if is_retryable(&e) && attempts < MAX_RETRIES => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

#### UC-3.2: Fallback Providers
**Pain Point**: Primary provider down, need backup provider
**Solution**: Try OpenAI, fall back to Anthropic if fails
**Priority**: High (availability)
**Example**:
```rust
async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse> {
    match f().await {
        Ok(resp) => Ok(resp),
        Err(e) if is_provider_error(&e) => {
            log::warn!("Primary provider failed, trying fallback");
            self.fallback_provider.complete(messages, tools).await
        }
        Err(e) => Err(e),
    }
}
```

#### UC-3.3: Response Caching
**Pain Point**: Identical requests sent multiple times, wasting cost
**Solution**: Cache responses by (messages, tools) hash
**Priority**: Medium (cost optimization)
**Example**:
```rust
async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse> {
    let cache_key = self.compute_cache_key(&messages, &tools);
    if let Some(cached) = self.cache.get(&cache_key).await {
        return Ok(cached);
    }
    let response = f().await?;
    self.cache.set(cache_key, response.clone()).await;
    Ok(response)
}
```

#### UC-3.4: Telemetry / Logging
**Pain Point**: Can't debug performance issues, track token usage
**Solution**: Log latency, tokens, cost for every call
**Priority**: High (observability)
**Example**:
```rust
async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse> {
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    match &result {
        Ok(resp) => {
            metrics::histogram!("model_call.latency", duration);
            metrics::counter!("model_call.success", 1);
        }
        Err(e) => {
            metrics::counter!("model_call.error", 1);
            log::error!("Model call failed: {}", e);
        }
    }
    result
}
```

#### UC-3.5: Rate Limiting
**Pain Point**: Hitting provider rate limits, need to queue/throttle
**Solution**: Use semaphore or token bucket to limit concurrent calls
**Priority**: Medium (production scaling)
**Example**:
```rust
async fn wrap_model_call<F, Fut>(&self, f: F) -> ProviderResult<ProviderResponse> {
    let _permit = self.rate_limiter.acquire().await?;
    f().await
}
```

### Implementation Priority

**Layer 2**: None (mock provider doesn't fail)
**Layer 3**: Retry logic, telemetry (high priority for production)
**Layer 4**: Fallback providers, caching, rate limiting (enterprise)

---

## Hook 4: `after_model`

**Called**: After model responds, before tool execution
**Signature**: `async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction>`
**Default**: Continue (proceed normally)

### Use Cases

#### UC-4.1: Human-in-the-Loop Approval
**Pain Point**: Agent may take destructive actions, need human approval
**Solution**: Pause for approval before executing tool calls
**Priority**: High (safety-critical for production)
**Example**:
```rust
async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction> {
    if let ProviderResponse::ToolCalls(calls) = response {
        if calls.iter().any(|c| is_dangerous_tool(&c.name)) {
            let approved = self.request_human_approval(calls).await?;
            if !approved {
                return Ok(HookAction::Reject("User rejected action".into()));
            }
        }
    }
    Ok(HookAction::Continue)
}
```

#### UC-4.2: Safety Validation
**Pain Point**: Model may generate harmful, biased, or inappropriate content
**Solution**: Check response against safety policies, reject if violated
**Priority**: High (compliance, safety)
**Example**:
```rust
async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction> {
    if let ProviderResponse::Text(text) = response {
        if self.safety_validator.is_harmful(text).await? {
            return Ok(HookAction::Reject("Response failed safety check".into()));
        }
    }
    Ok(HookAction::Continue)
}
```

#### UC-4.3: Response Formatting
**Pain Point**: Need consistent output format (JSON, markdown, etc.)
**Solution**: Parse and reformat model response
**Priority**: Low (usually handled in tools)
**Example**:
```rust
async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction> {
    if let ProviderResponse::Text(text) = response {
        let formatted = self.format_as_markdown(text);
        return Ok(HookAction::Modify(ProviderResponse::Text(formatted)));
    }
    Ok(HookAction::Continue)
}
```

#### UC-4.4: Content Filtering
**Pain Point**: Need to remove PII, profanity, or sensitive info from responses
**Solution**: Filter response content before returning
**Priority**: Medium (compliance)
**Example**:
```rust
async fn after_model(&self, response: &ProviderResponse) -> Result<HookAction> {
    if let ProviderResponse::Text(text) = response {
        let filtered = self.pii_filter.remove_sensitive_data(text)?;
        return Ok(HookAction::Modify(ProviderResponse::Text(filtered)));
    }
    Ok(HookAction::Continue)
}
```

### Implementation Priority

**Layer 2**: None (simple agents don't need approval)
**Layer 3**: HITL approval, safety validation (high priority for production)
**Layer 4**: V1 async HITL import, sophisticated validation (enterprise)

---

## Hook 5: `wrap_tool_call`

**Called**: Wraps every tool execution
**Signature**: `async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult`
**Default**: Direct call (`f().await`)

### Use Cases

#### UC-5.1: Tool Retry Logic
**Pain Point**: Tools fail transiently (network, file locks), need retry
**Solution**: Retry failed tool calls with backoff
**Priority**: High (tool reliability)
**Example**:
```rust
async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult
where F: Fn() -> Fut, Fut: Future<Output = ToolResult>
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) && attempts < MAX_RETRIES => {
                attempts += 1;
                log::warn!("Tool {} failed (attempt {}), retrying", name, attempts);
                tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

#### UC-5.2: Permission Checks
**Pain Point**: Need to ensure agent allowed to use certain tools
**Solution**: Check permissions before executing, reject if unauthorized
**Priority**: Medium (security)
**Example**:
```rust
async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult {
    if !self.permissions.is_allowed(agent_id, name).await? {
        return Err(format!("Tool '{}' not authorized", name).into());
    }
    f().await
}
```

#### UC-5.3: Audit Logging
**Pain Point**: Need to track all tool invocations for compliance/debugging
**Solution**: Log tool name, arguments, result, duration
**Priority**: High (observability, compliance)
**Example**:
```rust
async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult {
    let start = Instant::now();
    log::info!("Executing tool: {}", name);

    let result = f().await;
    let duration = start.elapsed();

    match &result {
        Ok(output) => {
            log::info!("Tool {} completed in {:?}: {}", name, duration, output);
            self.audit_log.record_success(name, duration).await;
        }
        Err(e) => {
            log::error!("Tool {} failed: {}", name, e);
            self.audit_log.record_failure(name, e).await;
        }
    }
    result
}
```

#### UC-5.4: Dry-Run Mode
**Pain Point**: Want to test agent without actually executing tools
**Solution**: Intercept tool calls, return mock results
**Priority**: Low (testing/debugging feature)
**Example**:
```rust
async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult {
    if self.dry_run_mode {
        log::info!("DRY RUN: Would execute tool '{}'", name);
        return Ok(format!("[DRY RUN] Tool '{}' would execute here", name));
    }
    f().await
}
```

#### UC-5.5: Result Validation
**Pain Point**: Tools may return invalid data that breaks agent
**Solution**: Validate tool output meets expected schema
**Priority**: Medium (robustness)
**Example**:
```rust
async fn wrap_tool_call<F, Fut>(&self, name: &str, f: F) -> ToolResult {
    let result = f().await?;
    if !self.validator.validate_tool_output(name, &result) {
        return Err(format!("Tool '{}' returned invalid output", name).into());
    }
    Ok(result)
}
```

### Implementation Priority

**Layer 2**: None (simple tools don't fail often)
**Layer 3**: Retry logic, audit logging (high priority for production)
**Layer 4**: Permission checks, dry-run mode, validation (enterprise)

---

## Hook 6: `after_agent`

**Called**: After agent completes execution
**Signature**: `async fn after_agent(&self, result: &str) -> Result<String>`
**Default**: Passthrough (returns result unchanged)

### Use Cases

#### UC-6.1: Result Persistence
**Pain Point**: Need to save agent results to database, file, or API
**Solution**: Persist result before returning to user
**Priority**: Medium (data management)
**Example**:
```rust
async fn after_agent(&self, result: &str) -> Result<String> {
    self.db.save_result(session_id, result).await?;
    Ok(result.to_string())
}
```

#### UC-6.2: Notification / Alerting
**Pain Point**: Need to notify on agent completion (email, webhook, Slack)
**Solution**: Send notification with result summary
**Priority**: Low (nice to have)
**Example**:
```rust
async fn after_agent(&self, result: &str) -> Result<String> {
    if self.should_notify(result) {
        self.notify_user(user_email, result).await?;
    }
    Ok(result.to_string())
}
```

#### UC-6.3: Metrics Collection
**Pain Point**: Need to track success/failure rates, latency, quality
**Solution**: Record metrics for monitoring dashboards
**Priority**: High (observability)
**Example**:
```rust
async fn after_agent(&self, result: &str) -> Result<String> {
    metrics::counter!("agent.completions", 1);
    metrics::histogram!("agent.result_length", result.len() as f64);

    if self.quality_scorer.score(result).await? > THRESHOLD {
        metrics::counter!("agent.high_quality_results", 1);
    }
    Ok(result.to_string())
}
```

#### UC-6.4: Post-Processing
**Pain Point**: Need to transform result to specific format (PDF, email, JSON)
**Solution**: Convert result to desired output format
**Priority**: Medium (UX improvement)
**Example**:
```rust
async fn after_agent(&self, result: &str) -> Result<String> {
    match self.output_format {
        OutputFormat::Json => Ok(json!({"result": result}).to_string()),
        OutputFormat::Markdown => Ok(format!("# Result\n\n{}", result)),
        OutputFormat::Plain => Ok(result.to_string()),
    }
}
```

### Implementation Priority

**Layer 2**: None (simple agents just return result)
**Layer 3**: Metrics collection (high priority for production monitoring)
**Layer 4**: Result persistence, notifications, post-processing (enterprise)

---

## Priority Matrix

| Hook | High Priority Use Cases | Layer | Trigger |
|------|------------------------|-------|---------|
| before_agent | Rate limiting, input validation | 3 | Production deployment, bad inputs |
| before_model | Context window management | 3 | Long conversations, token limits |
| wrap_model_call | Retry logic, telemetry | 3 | API reliability issues, debugging pain |
| after_model | HITL approval, safety validation | 3 | Destructive actions, compliance requirements |
| wrap_tool_call | Retry logic, audit logging | 3 | Tool failures, compliance requirements |
| after_agent | Metrics collection | 3 | Production monitoring needs |

## Implementation Roadmap

### Layer 2.5 (Week 4)
- **Deliverable**: Hook architecture only (no implementations)
- Define all 6 hooks in `AgentLifecycle` trait
- Integration in `Agent::run()` method
- Default passthrough implementations

### Layer 3 (Month 2+)
**Implement when pain validated:**

**Priority 1** (likely first):
- `wrap_model_call` with retry + telemetry
- `wrap_tool_call` with retry + audit logging
- `before_agent` with input validation

**Priority 2** (if needed):
- `before_model` with context trimming
- `after_model` with safety validation
- `after_agent` with metrics

### Layer 4 (Q1 2026)
**Import from V1 archive:**
- V1 async HITL → `after_model` hook
- V1 MAPE-K monitoring → all hooks with telemetry
- V1 Tower middleware → composable hook chains

## Relationships
- **Supports**: `decisions/lifecycle-hook-architecture.md` - architecture decision
- **Informs**: `planning/roadmap.md` - Layer 3/4 feature planning
- **Sources**: External production experience, LangChain V1 validation
- **Guides**: Future plugin/middleware development

## Metadata
- **Created**: 2025-10-16
- **Source**: Production experience + LangChain V1 patterns
- **Status**: Living document (add use cases as discovered)
- **Next Review**: Layer 3 planning (Month 2)

## Change History
- 2025-10-16: Created comprehensive use case catalog with priority matrix
