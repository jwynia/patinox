# ADR-001: OpenAI API Format Adoption for LMStudio Provider

## Status
**ACCEPTED** - 2025-08-25

## Context
LMStudio provides OpenAI-compatible API endpoints, offering a choice between implementing custom LMStudio-specific patterns (like Ollama) or leveraging the standard OpenAI API format.

## Decision
We will implement the LMStudio provider using **OpenAI-compatible API format** rather than creating LMStudio-specific patterns.

## Rationale

### Advantages of OpenAI API Format

**1. Standardization Benefits**
- Well-documented and widely understood API format
- Consistent with industry standards
- Reduces learning curve for developers familiar with OpenAI API

**2. Code Reuse Opportunities**
- Can leverage request/response structures from existing OpenAI provider
- Established patterns for error handling and validation
- Common JSON schema definitions already exist

**3. Maintenance Advantages**  
- Standard format is stable and mature
- Less custom code means fewer bugs
- Easier testing with existing OpenAI tooling

**4. Integration Benefits**
- Compatible with OpenAI client libraries for testing
- Familiar patterns for developers
- Consistent API experience across cloud and local providers

### Comparison with Alternatives

| Approach | Pros | Cons | Effort |
|----------|------|------|--------|
| **OpenAI Format** ✅ | Standard, reusable, documented | Less LMStudio-specific | Medium |
| Custom Format | LMStudio-optimized | More code, maintenance | High |
| Hybrid Approach | Flexible | Complex, inconsistent | High |

## Implementation Implications

### API Endpoints to Implement
- **Model Listing**: `GET /v1/models` (OpenAI format)
- **Chat Completions**: `POST /v1/chat/completions` (OpenAI format)
- **Embeddings** (Optional): `POST /v1/embeddings` (OpenAI format)

### Request/Response Handling
- Use OpenAI-compatible request structures
- Leverage existing JSON serialization/deserialization
- Apply standard OpenAI error response patterns

### Code Structure Impact
```rust
// Reuse OpenAI request/response types where possible
use crate::provider::openai::{OpenAICompletionRequest, OpenAICompletionResponse};

// Custom transformation only where necessary
impl LMStudioProvider {
    fn convert_to_openai_format(&self, request: CompletionRequest) -> OpenAICompletionRequest {
        // Direct mapping with minimal transformation
    }
}
```

### Testing Strategy
- Can use OpenAI API testing patterns
- Standard mock responses from OpenAI format
- Compatibility testing with OpenAI client libraries

## Risks and Mitigations

### Risk 1: LMStudio API Deviations
**Risk**: LMStudio might have minor deviations from OpenAI API
**Mitigation**: Comprehensive testing with actual LMStudio service, document any deviations

### Risk 2: Limited LMStudio Features
**Risk**: Some LMStudio-specific features might not fit OpenAI format
**Mitigation**: Start with core features, extend as needed with LMStudio-specific handling

### Risk 3: OpenAI Format Changes
**Risk**: OpenAI API format evolution might require updates
**Mitigation**: Use versioned endpoints, test against stable API versions

## Success Criteria
- [ ] LMStudio provider uses standard OpenAI API endpoints
- [ ] Request/response processing reuses existing OpenAI patterns where possible
- [ ] API format is compatible with standard OpenAI tooling
- [ ] Implementation effort is reduced compared to custom format approach

## Implementation Guidelines

### Code Organization
- Place shared OpenAI structures in common module
- Use traits to abstract OpenAI format handling
- Keep LMStudio-specific logic minimal and isolated

### Error Handling
- Map OpenAI error responses to ProviderError types
- Use established HTTP error mapping patterns
- Provide clear error messages for local service issues

### Testing Requirements
- Test compatibility with OpenAI mock responses
- Verify LMStudio-specific behavior where it deviates
- Include integration tests with actual LMStudio service

## Alternatives Considered

### Alternative 1: Custom LMStudio Format
**Rejected** - Would require more development effort and maintenance without significant benefits

### Alternative 2: Hybrid Approach  
**Rejected** - Would create inconsistency and complexity without clear advantages

### Alternative 3: Direct LMStudio API Binding
**Rejected** - Would tightly couple to LMStudio-specific features, reducing standardization benefits

## References
- OpenAI API Documentation: https://platform.openai.com/docs/api-reference
- LMStudio Documentation: (Local service, OpenAI-compatible)
- Existing OpenAI Provider Implementation: `src/provider/openai.rs`
- TDD Provider Implementation Pattern: `context-network/implementation/tdd-provider-implementation-pattern.md`

---

**Decision Made By**: Planning Team  
**Implementation Impact**: Medium - requires careful pattern reuse and testing  
**Review Date**: After implementation completion to validate decision effectiveness