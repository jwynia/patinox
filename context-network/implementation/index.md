# Implementation Patterns and Guides

This directory contains proven patterns and methodologies for implementing Patinox framework components, extracted from successful implementations and validated through real-world usage.

## Provider Implementation

### Core Methodology
- **[TDD Provider Implementation Pattern](./tdd-provider-implementation-pattern.md)** - Comprehensive test-driven development approach for provider implementations
  - *Proven through*: Ollama provider implementation (16 tests, 100% pass rate)
  - *Applies to*: All future provider implementations
  - *Key benefit*: Reduces development time and ensures comprehensive error handling

### Technical Patterns
- **[Provider HTTP Error Mapping Guide](./provider-http-error-mapping-guide.md)** - Standardized error handling across all providers
  - *Ensures*: Consistent user experience and maintainable error handling
  - *Applies to*: Cloud providers, local providers, router providers
  - *Integration*: Works with existing ProviderError framework

### Specialized Patterns
- **[Local Provider Integration Patterns](../discovery/2025-08-23-001-local-provider-integration-patterns.md)** - Patterns specific to local service integration
  - *Discovered*: During Ollama implementation (2025-08-23)
  - *Key insights*: Service availability, error handling, testing strategies
  - *Next application*: LMStudio provider implementation

## Memory Management
- **[Memory Management Implementation Record](./memory-management-implementation-record.md)** - RAII patterns and resource cleanup strategies
- **[LLM Provider Implementation Record](./llm-provider-implementation-record.md)** - Provider abstraction and multi-provider support

## Implementation Records
Implementation records document the process, decisions, and learnings from completed implementations:

- **Memory Management**: Resource cleanup and RAII patterns
- **LLM Provider Abstraction**: Multi-provider framework design
- **OpenRouter Provider**: Multi-provider routing implementation
- **Anthropic Provider**: Claude model integration with security patterns

## Navigation Guide

### For New Provider Implementation
1. Start with **[TDD Provider Implementation Pattern](./tdd-provider-implementation-pattern.md)** for methodology
2. Follow **[Provider HTTP Error Mapping Guide](./provider-http-error-mapping-guide.md)** for consistent error handling
3. If implementing local provider, review **[Local Provider Integration Patterns](../discovery/2025-08-23-001-local-provider-integration-patterns.md)**
4. Reference existing provider implementations for concrete examples

### For Error Handling Standardization
1. Review **[Provider HTTP Error Mapping Guide](./provider-http-error-mapping-guide.md)** for patterns
2. Check existing implementations for consistent application
3. Ensure all error types map to appropriate ProviderError variants

### For Understanding Architectural Decisions
1. Check implementation records for specific components
2. Review discovery records for insights and lessons learned
3. Follow decision trails in planning documents

---

*This index is maintained to ensure implementation patterns are discoverable and applied consistently across the framework.*