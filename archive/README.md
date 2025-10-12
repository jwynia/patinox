# Archive Directory

This directory contains archived code and artifacts from previous phases of the Patinox project.

## Purpose

Keeping the project root clean and focused by moving inactive/research code to a dedicated archive location.

## Contents

### `/examples-v1-enterprise/`
Example code from V1 (sophisticated-first architecture). Contains:
- `typestate_examples.rs` - TypeState pattern demonstrations

**Status**: Archived October 12, 2025
**Reason**: V1 implementation archived; examples preserved for reference when building Layer 4 (Enterprise Features)

### `/src-v1-enterprise/`
Complete V1 source code implementing sophisticated-first architecture. Contains:
- Core traits (Agent, Tool, Validator, Monitor)
- Provider implementations (OpenAI, Anthropic, OpenRouter, Ollama, LMStudio)
- Validation system (anti-jailbreak, hallucination detection)
- Memory management (registry, resource pooling)
- TypeState builder patterns

**Status**: Archived October 12, 2025
**Reason**: Strategic pivot to minimal-first (V2). Preserved as research and import source for Layer 4.
**Size**: ~4,000 lines of production-quality Rust
**Value**: Proven patterns for enterprise features

### `/tests-v1-enterprise/`
Comprehensive test suite for V1 implementation. Contains:
- Unit tests for all modules
- Integration tests for providers
- Validation pipeline tests
- Memory management tests
- Property-based tests

**Status**: Archived October 12, 2025
**Reason**: Tests specific to V1 architecture
**Value**: Test patterns and approaches to reuse in V2

## Git References

All archived code is also preserved in git:
- **Branch**: `archive/patinox-v1-sophisticated-first`
- **Tag**: `v1-research-phase`
- **Context Network**: `context-network/archive/v1-research/`

## Usage

### When to Reference This Archive

1. **Building Layer 4** (Enterprise Features):
   - Import provider implementations
   - Adapt validation logic
   - Reuse monitoring patterns

2. **Understanding V1 Research**:
   - Study enterprise architecture decisions
   - Review comprehensive trait system
   - Learn from V1 approach

3. **Test Pattern Reference**:
   - Proven testing strategies
   - Integration test patterns
   - Mock/wiremock usage

### Import Strategy

When importing from archive:
1. Copy needed code to appropriate V2 location
2. Simplify for V2's minimal-first approach
3. Adapt to current architecture
4. Test thoroughly
5. Keep archive untouched (reference only)

## Folder Organization

```
archive/
├── README.md                 # This file
├── examples-v1-enterprise/   # V1 example code
├── src-v1-enterprise/        # V1 source implementation (~4K lines)
└── tests-v1-enterprise/      # V1 test suite
```

## Related Documentation

- Strategic reset decision: `context-network/decisions/v2_strategic_reset.md`
- V1 research overview: `context-network/archive/v1-research/README.md`
- V2 architecture: See main `README.md`

---

**Last Updated**: October 12, 2025
**Status**: Stable archive, reference only
