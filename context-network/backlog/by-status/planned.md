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

## Blocked: Waiting on Real Usage Data

### Plugin Development (All Conditional)

**Blocker**: Need to build 2-3 real agents first to identify actual pain points

**Potential plugins** (may or may not be needed):
- Memory Plugin - *IF agents forget context*
- Discovery Plugin - *IF file/directory exploration is painful*
- Config Plugin - *IF hardcoding becomes frustrating*
- Resource Plugin - *IF rate limits/costs become painful*

**How they become ready**:
1. Build real agents (Week 2, Days 3-5)
2. Document every "I wish it had X" moment
3. Analyze pain points by frequency + severity (Day 6)
4. Highest pain point gets designed (Day 7)
5. Move to ready.md for Week 3 implementation

**See**: [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for pain point analysis framework

---

## Blocked: Waiting on Provider Integration

### Real Agent Development

**Blocker**: Need real LLM provider working first (Week 2, Phase 1)

**Candidates** (pick 2 after provider works):
- **File Processor Agent** - Process text files with LLM analysis
- **Git Helper Agent** - Analyze repos, suggest improvements
- **Documentation Generator** - Read code, generate docs

**How they become ready**:
1. Real provider integration completes
2. Pick 2 agents based on immediate utility
3. Groom with specific tools needed
4. Move to ready.md

**Estimated**: Ready by Week 2, Day 3

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
**Last updated by**: V2 Context Recovery
**Total planned tasks**: 0 specific tasks (principles documented)
**V2 Phase**: Layer 2 - Week 2, awaiting usage data

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
