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

**Context**:
- ✅ Layer 1 (Minimal Agent) completed October 12, 2025
- ✅ Week 2 Phase 1 (Real Provider) completed October 13, 2025
- ⏳ Week 2 Phase 2 (Build Real Agents) - NOW READY

**See**: [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for full Week 2 strategy.

---

## Week 2 Phase 2: Build Real Agents (Days 3-5)

### V2-AGENT-001: Build File Processor Agent

**One-liner**: Create an agent that processes text files with LLM analysis to validate V2 framework

**Priority**: High (Week 2 Phase 2 goal #1)
**Effort**: 2-4 hours
**Branch**: `feat/v2-file-processor-agent`

**Why This Matters**: First real-world usage of V2 agent framework. Will reveal what plugins are actually needed vs. theoretically useful.

**Acceptance Criteria**:
- [ ] Agent reads files from command line arguments
- [ ] Agent uses OpenAI provider to analyze content
- [ ] Agent provides useful output (summary, analysis, insights)
- [ ] Used for actual work on real files (not just demo)
- [ ] **All pain points documented** with frequency and severity

**Implementation Plan**:
1. Create `examples/file_processor.rs` using V2 agent API
2. Add tools: `read_file`, `summarize`, `analyze`
3. Test with real markdown, code, and log files
4. Document every "I wish it had X" moment in pain point log
5. Track time spent working around limitations

**Pain Point Documentation**:
Create `context-network/records/pain-points-file-processor-[date].md`:
```markdown
## Pain Point: [Title]
- Frequency: [How often hit]
- Severity: [1-5, 5=blocker]
- Workaround: [What you did instead]
- Time cost: [Minutes/hours wasted]
- Potential solution: [Plugin/feature needed]
```

**Files to Create**:
- `examples/file_processor.rs` - Agent implementation
- `context-network/records/pain-points-file-processor-2025-10-13.md` - Pain log

---

### V2-AGENT-002: Build Documentation Generator Agent

**One-liner**: Create an agent that reads code and generates documentation to identify parsing/template needs

**Priority**: High (Week 2 Phase 2 goal #2)
**Effort**: 3-5 hours
**Branch**: `feat/v2-doc-generator-agent`

**Why This Matters**: Second real usage case. Will reveal if AST parsing, template rendering, or multi-file plugins are needed.

**Acceptance Criteria**:
- [ ] Agent reads Rust source files
- [ ] Agent generates useful markdown documentation
- [ ] Output quality is good enough to actually use
- [ ] Pain points documented (parsing? templates? discovery?)
- [ ] Used on patinox codebase itself

**Implementation Plan**:
1. Create `examples/doc_generator.rs`
2. Start simple: read file → LLM → markdown
3. Add tools as pain demands: `parse_rust`, `format_markdown`, `write_file`
4. Test on patinox modules (agent.rs, tool.rs, provider.rs)
5. Document every limitation hit

**Expected Pain Points**:
- Code parsing without syn/AST support
- Multi-file processing (discovery?)
- Output templating needs
- Context window limits (memory?)

**Files to Create**:
- `examples/doc_generator.rs`
- `context-network/records/pain-points-doc-generator-2025-10-13.md`

---

## Metadata

**Last updated**: 2025-10-13 (Grooming session)
**Last updated by**: Backlog grooming after V2 Phase 1 completion
**Total ready tasks**: 2
**V2 Phase**: Layer 2 - Week 2, Phase 2 (Build Real Agents)

## Notes

This is a fresh V2 backlog. Previous V1 refinement tasks (streaming optimization, validation improvements, etc.) were deleted to avoid "pink elephant" effect.

**V2 Principle**: Tasks are added **only after pain is felt** through real usage. We don't plan sophistication in advance.

**Next tasks emerge from**:
- Building 2-3 real agents (Week 2, Days 3-5)
- Documenting pain points during building
- Pain point analysis (Week 2, Day 6)

See [planning/v2-week-2-plan.md](../../planning/v2-week-2-plan.md) for the emergence strategy.
