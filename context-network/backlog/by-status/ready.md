# Ready for Implementation

Tasks that are fully groomed, have no blockers, and are ready to be worked on immediately.

## What Makes a Task "Ready"?

A task is ready when:
- ✅ Acceptance criteria are clearly defined
- ✅ All dependencies are completed or resolved
- ✅ No open questions or decisions needed
- ✅ Estimated effort is reasonable
- ✅ Implementation approach is documented
- ✅ Someone can start within 5 minutes of reading

## Current Phase: V2 Layer 2 - Real Usage & Pain-Driven Plugins

**Context**: Layer 1 (Minimal Agent) completed October 12, 2025. Now building real agents to identify what plugins are actually needed.

**See**: [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for full Week 2 strategy.

---

## Week 2 Phase 1: Real Provider Integration (Days 1-2)

### Task: Integrate Real LLM Provider
**Priority**: Critical | **Effort**: 4-6 hours | **Owner**: Unassigned

**Goal**: Replace mock provider with real LLM (OpenAI or Anthropic) so agents can do actual work.

**Acceptance Criteria**:
- [ ] Choose provider approach (async-openai crate vs import from V1 vs build minimal)
- [ ] Add async runtime (tokio)
- [ ] Implement real provider in `src/provider.rs`
- [ ] Add API key configuration (environment variables)
- [ ] Update `examples/hello_agent.rs` to use real provider
- [ ] Example compiles, runs, makes real API call
- [ ] Error handling for API failures (network, auth, rate limits)

**Implementation Options**:
1. **async-openai crate** (recommended) - maintained, immediate integration
2. **Import from V1 archive** - battle-tested, supports 5 providers
3. **Build minimal** - only what's needed, stays minimal

**First Steps**:
1. Research: Quick eval of async-openai crate capabilities
2. Decision: Choose approach based on simplicity vs completeness
3. Add dependencies to Cargo.toml
4. Implement Provider trait for chosen approach
5. Test with hello_agent

**Files to modify**:
- `Cargo.toml` - Add tokio, provider crate/code
- `src/provider.rs` - Real implementation
- `examples/hello_agent.rs` - Update to async
- `.env.example` - API key template

**Blockers**: None

---

## Week 2 Phase 2: Build Real Agents (Days 3-5)

*Tasks will be added after Phase 1 completes and we can identify actual use cases*

**Candidates** (not yet groomed):
- File processor agent
- Git helper agent
- Documentation generator agent

**Approach**: Pick 2 based on immediate utility, build them, document every pain point.

---

## Metadata

**Last updated**: 2025-10-13
**Last updated by**: V2 Context Recovery
**Total ready tasks**: 1 (real provider integration)
**V2 Phase**: Layer 2 - Week 2, Phase 1

## Notes

This is a fresh V2 backlog. Previous V1 refinement tasks (streaming optimization, validation improvements, etc.) were deleted to avoid "pink elephant" effect.

**V2 Principle**: Tasks are added **only after pain is felt** through real usage. We don't plan sophistication in advance.

**Next tasks emerge from**:
- Building 2-3 real agents (Week 2, Days 3-5)
- Documenting pain points during building
- Pain point analysis (Week 2, Day 6)

See [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for the emergence strategy.
