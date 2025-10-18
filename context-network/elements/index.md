# Elements Index

## Purpose
This document indexes all technical elements and components of the Patinox AI agent framework. Currently, detailed technical component documentation is distributed across other sections (architecture, planning, decisions) as part of the V2 minimal-first approach.

## Classification
- **Domain:** Project Structure
- **Stability:** Semi-stable
- **Abstraction:** Structural
- **Confidence:** Evolving

## Current Status

The elements section is currently **unpopulated** as part of the V2 strategic reset (October 12, 2025). The V2 minimal-first approach focuses on building working agents first, then documenting patterns that emerge from actual usage.

### V1 Element Documentation (Archived)
Comprehensive V1 element documentation has been archived to:
- **Location**: `context-network/archive/v1-research/elements/`
- **Status**: Reference only - V1 sophisticated-first approach
- **Import Strategy**: V1 components will be re-documented here as they are validated and imported into V2

## V2 Element Evolution Strategy

Elements will be added to this section based on pain-driven development:

1. **Build working agents** (Layer 1-2) - October 2025
2. **Identify common patterns** through real usage
3. **Document validated patterns** as elements here
4. **Import proven V1 components** when usage validates need (Layer 4)

## Related Documentation

Instead of this section, current V2 technical documentation is located in:

### Active V2 Documentation
- **Architecture Decisions**: `context-network/decisions/`
  - `v2_strategic_reset.md` - V2 minimal-first approach
  - `lifecycle-hook-architecture.md` - Layer 2.5 hooks design
- **Planning Documents**: `context-network/planning/`
  - `roadmap.md` - V2 layered architecture timeline
  - `v2-week-2-plan.md` - Real usage validation strategy
  - `v2-plugin-tool-context-design.md` - First plugin design
- **Implementation Records**: `context-network/records/`
  - Completion records for V2-AGENT-001, V2-AGENT-002
  - Pain point analysis from real agent development
  - Design validation records

### Structural Documentation (Available)
- **Dependencies**: `context-network/connections/dependencies.md`
- **Interfaces**: `context-network/connections/interfaces.md`
- **Architecture Overview**: Currently in planning docs, will migrate here as patterns stabilize

## Future Element Categories (Planned)

Based on V2 roadmap, this section will eventually contain:

### Layer 1-2 Elements (Current Focus)
- Agent Core (minimal ReACT loop)
- Tool System (function closures)
- Provider Abstraction (LLM integration)
- Plugin Architecture (pain-driven enhancements)

### Layer 3 Elements (November 2025+)
- Reasoning Patterns (plan-execute, reflexion)
- Multi-Agent Coordination
- Tool Composition

### Layer 4 Elements (Q1 2026+)
- MAPE-K Monitoring (from V1)
- Tower Validation (from V1)
- TypeState Patterns (from V1)
- Git-Based Evolution

## Navigation

### For Understanding Current V2 Architecture
1. Start with `context-network/decisions/v2_strategic_reset.md` for context
2. Review `context-network/planning/roadmap.md` for layered approach
3. Check `context-network/backlog/by-status/ready.md` for current implementation focus
4. See `context-network/records/` for what's been built and lessons learned

### For V1 Research Reference
1. Explore `context-network/archive/v1-research/elements/` for sophisticated-first documentation
2. Note: V1 is reference only, not for direct import until validated through V2 usage

## Related Sections
- [Foundation Index](../foundation/index.md) - Core project principles
- [Decisions Index](../decisions/index.md) - Architectural decisions
- [Planning Index](../planning/roadmap.md) - Development roadmap
- [Connections](../connections/dependencies.md) - Component dependencies
- [Processes](../processes/creation.md) - Development processes

## Parent Navigation
- **Parent:** [Context Network Discovery](../discovery.md)

## Metadata
- **Created:** 2025-10-17
- **Last Updated:** 2025-10-17
- **Updated By:** Context Network Audit Remediation (Recommendation #1)

## Change History
- 2025-10-17: Created index file to fix broken navigation from discovery.md; documented V2 minimal-first evolution strategy for elements section
