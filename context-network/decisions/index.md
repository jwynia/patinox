# Decisions Index

## Purpose
This document indexes all key architectural and project decisions that guide the development of the Patinox AI agent framework.

## Classification
- **Domain:** Project Governance
- **Stability:** Static
- **Abstraction:** Policy
- **Confidence:** Established

## Critical Decisions

### [Begin Coding Decision](begin_coding_decision.md) - 2025-08-18
**Status:** ACTIVE - Implementation Authorized

Formal authorization to begin implementation work after completing all architectural planning. This decision lifts the previous "no coding" restriction and enables development to proceed.

### [Architectural Decisions Resolved](architectural_decisions_resolved.md) - 2025-08-19
**Status:** Resolved

Comprehensive resolution of all major architectural decisions including the 8-crate structure, MAPE-K pattern adoption, async concurrency model, and technology stack selections.

### [Telemetry Debugging Approach](telemetry_debugging_approach.md) - 2025-08-18
**Status:** Active

Decision on how to implement comprehensive telemetry and debugging capabilities throughout the system for both development and production use.

## Historical Decisions

### [CRITICAL: No Coding Yet](CRITICAL_NO_CODING_YET.md) - Superseded
**Status:** SUPERSEDED by Begin Coding Decision

Originally established to prevent premature implementation while architectural decisions were being resolved. This restriction was formally lifted on 2025-08-18.

## Decision Process

### [Decision Template](decision_template.md)
Standard template for documenting new architectural and project decisions.

### Decision Guidelines
All major decisions should:
1. Follow the decision template format
2. Include clear rationale and alternatives considered
3. Specify implementation implications
4. Define success criteria where applicable
5. Include change history

## Decision Categories

### Architectural Decisions
- System structure and organization
- Technology and framework selections
- Design pattern adoptions
- Performance and scalability approaches

### Process Decisions
- Development workflows
- Quality assurance approaches
- Testing strategies
- Release management

### Technical Decisions
- Implementation techniques
- Library and dependency choices
- Configuration management
- Monitoring and observability

## Active Decision Impact

### Current Development Phase
Following the [Begin Coding Decision](begin_coding_decision.md), development is proceeding with:
- Implementation of foundational components
- Test-driven development approach
- Quality-first methodology
- Incremental feature delivery

### Key Constraints
Based on resolved architectural decisions:
- 8-crate workspace structure required
- MAPE-K pattern for agent architecture
- Async-first implementation approach
- Zero unsafe code in core abstractions

## Related Sections
- [Foundation](../foundation/) - Core principles that inform decisions
- [Planning](../planning/) - Implementation strategies based on decisions
- [Architecture](../elements/) - Technical components implementing decisions

## Navigation
- **Parent:** [Context Network Discovery](../discovery.md)
- **Related:** [Implementation Readiness Guide](../planning/implementation_readiness_guide.md)

## Metadata
- **Created:** 2025-09-18
- **Last Updated:** 2025-09-18
- **Updated By:** Context Network Audit Remediation

## Change History
- 2025-09-18: Initial creation of decisions index to organize project governance documents