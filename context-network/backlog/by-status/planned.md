# Planned Tasks

Tasks that have been identified but have dependencies or blockers preventing immediate work.

## What Makes a Task "Planned"?

A task is planned when:
- ✅ Requirements are understood
- ✅ Acceptance criteria could be defined
- ⏳ Waiting on prerequisite tasks
- ⏳ Waiting on decisions or information
- ⏳ Waiting on pain point validation

## Current Phase: V2 Layer 2 - Real Usage & Pain-Driven Plugins

**V2 Principle**: Most tasks stay "planned" until **real usage proves they're needed**. We don't build features speculatively.

---

## Blocked: Waiting on Plugin Design

### V2-PLUGIN-002: CLI Plugin Design

**Blocker**: Waiting on V2-PLUGIN-001 (Tool Context Helper) implementation
**Estimated unblock**: Week 3, after first plugin proven
**Priority**: Critical (Pain Score: 30/30)

**Problem Statement**:
Manual CLI argument parsing required for custom agent arguments beyond basic input. Every agent needs ~30 lines of boilerplate for argument handling.

**Validated Pain**: Hit on 100% of agents with custom arguments (V2-AGENT-001 file path, V2-AGENT-002 output flag)

**How this becomes ready**:
1. V2-PLUGIN-001 completed and validated
2. Plugin architecture patterns proven
3. Design follows established plugin integration patterns
4. Move to ready.md for Week 3+

---

### V2-PLUGIN-003: Discovery Plugin Design

**Blocker**: Waiting on V2-PLUGIN-001 and V2-PLUGIN-002 completion
**Estimated unblock**: Week 3+
**Priority**: High (Pain Score: 20)

**Problem Statement**:
Multi-file and batch processing requires manual file discovery and traversal logic. Common pattern for doc generation, code analysis, and data processing agents.

**Validated Pain**: Hit on doc generator agent (V2-AGENT-002) for multi-file documentation

**How this becomes ready**:
1. First two plugins proven
2. Plugin pattern established
3. Additional real usage validates need
4. Move to ready.md when design phase reached

---

## Future Layers (Not Yet Planned)

### Layer 3: Reasoning Patterns
**Trigger**: When simple ReACT loop proves insufficient
**Examples**: Plan-Execute, Reflexion, Multi-Agent
**Timeline**: November 2025+

### Layer 4: Enterprise Features
**Trigger**: When Layer 1-3 proven and enterprise needs validated
**Source**: Import from V1 archive
**Examples**: MAPE-K monitoring, Tower validation, TypeState
**Timeline**: Q1 2026+

---

## Metadata

**Last updated**: 2025-10-13
**Last updated by**: `/sync` command - post-Phase 3 planning update
**Total planned tasks**: 2 plugin designs (CLI Plugin, Discovery Plugin)
**V2 Phase**: Layer 2 - Week 2-3, plugin sequencing

## Notes

**V2 Approach**: This file documents *what we're waiting for* rather than *what we'll build*. Tasks emerge from pain, not from planning sophistication in advance.

**Anti-pattern avoided**: Long list of "future enhancements" that pull attention toward sophistication before it's validated.

**How tasks get added**:
1. Real usage reveals pain point
2. Pain point analyzed (frequency × severity)
3. Solution designed (minimal scope)
4. Task created with specific acceptance criteria
5. Moved to ready.md when dependencies clear

This is the "minimal-first" approach in practice.
