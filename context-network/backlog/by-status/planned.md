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

## Sequenced: Plugin Architecture Build-Out

### V2-PLUGIN-002: CLI Plugin Design

**Sequencing**: Follows V2-PLUGIN-001 (Tool Context Helper)
**Why sequenced**: Plugin trait architecture must be established first
**Priority**: Critical (Pain Score: 30/30)

**Problem Statement**:
Manual CLI argument parsing required for custom agent arguments beyond basic input. Every agent needs ~30 lines of boilerplate for argument handling.

**Validated Pain**: Hit on 100% of agents with custom arguments (V2-AGENT-001 file path, V2-AGENT-002 output flag)

**How this becomes ready**:
1. V2-PLUGIN-001 design completed (establishes plugin trait patterns)
2. V2-PLUGIN-001 implementation validated (proves builder integration)
3. CLI plugin design follows established patterns
4. Move to ready.md

**Prep Work Available Now**:
- Document existing CLI patterns from both agents
- Research CLI parsing libraries (clap, structopt)
- Draft requirements for CLI plugin API

---

### V2-PLUGIN-003: Discovery Plugin Design

**Sequencing**: Follows V2-PLUGIN-001 and V2-PLUGIN-002
**Why sequenced**: Need plugin pattern stability before third plugin
**Priority**: High (Pain Score: 20)

**Problem Statement**:
Multi-file and batch processing requires manual file discovery and traversal logic. Common pattern for doc generation, code analysis, and data processing agents.

**Validated Pain**: Hit on doc generator agent (V2-AGENT-002) for multi-file documentation
- Frequency: 2 agents likely
- Severity: 5 (annoyance, not blocker)

**How this becomes ready**:
1. First two plugins (Tool Context, CLI) proven and validated
2. Plugin architecture stable and documented
3. Additional usage validates multi-file processing need
4. Design follows established patterns
5. Move to ready.md

**Prep Work Available Now**:
- Document multi-file use cases
- Research file discovery patterns
- Sketch API for glob patterns, directory traversal

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

**Last updated**: 2025-10-13 (Grooming session)
**Last updated by**: `/groom` command
**Total planned tasks**: 2 plugin designs (CLI Plugin, Discovery Plugin)
**V2 Phase**: Layer 2 - Plugin architecture build-out

## Grooming Insights

**Sequencing Strategy**: Build plugins one-at-a-time to establish patterns
1. **V2-PLUGIN-001** (Tool Context) - defines plugin trait architecture
2. **V2-PLUGIN-002** (CLI) - validates plugin patterns work for different domains
3. **V2-PLUGIN-003** (Discovery) - confirms pattern stability

**Why Not Parallel**: Plugin trait design decisions in V2-PLUGIN-001 will inform all subsequent plugins. Building sequentially prevents rework.

**Deferred Improvements**: 3 code quality tasks (REFACTOR-001, ARCH-001, TEST-001) waiting for plugin architecture stability

## Notes

**V2 Approach**: This file documents *sequencing* rather than *speculation*. Tasks emerge from pain, not from planning sophistication in advance.

**Anti-pattern avoided**: Long list of "future enhancements" that pull attention toward sophistication before it's validated.

**How tasks flow**:
1. Real usage reveals pain point
2. Pain point analyzed (frequency × severity)
3. Solution designed (minimal scope)
4. Task created with specific acceptance criteria
5. Moved to ready.md when sequencing allows

This is the "minimal-first" approach in practice.
