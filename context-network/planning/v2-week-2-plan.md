# V2 Week 2+: Real Usage & Plugin Layer

## Purpose
Plan the next phase of V2 development focused on real-world usage and pain-driven plugin development.

## Classification
- **Domain:** Planning
- **Stability:** Dynamic (evolves with usage)
- **Abstraction:** Implementation Plan
- **Confidence:** Medium (depends on Week 1 learnings)

## Current State (End of Week 1)

### What We Have ✅
- Working minimal agent core (~200 lines)
- Builder pattern API
- Mock provider for testing
- CLI interface
- One working example (hello_agent)

### What We're Missing ⏳
- Real LLM integration (mock only)
- Actual usage (no production agents yet)
- Pain points data (haven't felt limitations)
- Plugin architecture (not needed yet)

## Week 2 Goals

### Primary Goal: **Validate Through Real Usage**

Build 2-3 agents for actual tasks and let pain drive features

### Success Criteria
1. ✅ At least 2 agents built for real tasks
2. ✅ Real LLM provider integrated (not mock)
3. ✅ Pain points documented (what's missing)
4. ✅ Decision on first plugin to build

## Phase 1: Real Provider Integration (Days 1-2)

### Objective
Replace mock provider with real LLM integration

### Options for Provider Source

**Option A: Import from V1 Archive (Recommended)**
- Pros: Battle-tested, comprehensive, already supports 5 providers
- Cons: May be over-engineered for minimal needs
- Location: `archive/src-v1-enterprise/provider/`
- Files to import:
  - `provider/types.rs` - Core types
  - `provider/openai.rs` or `provider/anthropic.rs` - One provider
  - `provider/error.rs` - Error handling

**Option B: Build Minimal from Scratch**
- Pros: Only what's needed, maintains minimal philosophy
- Cons: Rebuilds tested code, slower to real usage
- Approach: Simple HTTP client to OpenAI/Anthropic API

**Option C: Use Existing Crate (e.g., async-openai)**
- Pros: Maintained by community, immediate integration
- Cons: External dependency, less control
- Consideration: Aligns with "use what exists" principle

### Recommendation
**Start with Option C (async-openai)**, fallback to Option A if insufficient

### Acceptance Criteria
- [ ] Real provider integrated (OpenAI or Anthropic)
- [ ] hello_agent works with real LLM
- [ ] API key configuration added
- [ ] Error handling for API failures
- [ ] Async runtime added (tokio)

### Implementation Notes
- Add `tokio` for async runtime
- Add `async-openai` or similar for provider
- Update `provider.rs` with real implementation
- Update examples to use real provider
- Add configuration for API keys (env vars)

## Phase 2: Build Real Agents (Days 3-5)

### Objective
Build agents for actual tasks, document pain points

### Candidate Agents

**Agent 1: File Processor**
- **Task**: Process text files with LLM analysis
- **Tools needed**: read_file, write_file, list_directory
- **Pain points expected**: File I/O, error handling, streaming
- **Learning**: Do we need file system plugins?

**Agent 2: Git Helper**
- **Task**: Analyze git repos, suggest improvements
- **Tools needed**: git_status, git_diff, read_file
- **Pain points expected**: Command execution, parsing output
- **Learning**: Do we need process execution plugin?

**Agent 3: Documentation Generator**
- **Task**: Read code, generate documentation
- **Tools needed**: parse_rust, generate_markdown, write_file
- **Pain points expected**: Code analysis, template rendering
- **Learning**: Do we need AST parsing plugin?

### Pick 2 and Build
- Choose based on immediate utility
- Build minimal version first
- Document every "I wish it had X" moment
- Track time spent on workarounds

### Acceptance Criteria
- [ ] 2 agents completed and functional
- [ ] Used for actual work (not just demos)
- [ ] Pain points documented with specifics
- [ ] Tool patterns identified

## Phase 3: Pain Point Analysis (Day 6)

### Objective
Analyze real usage to prioritize plugin development

### Analysis Framework

**For Each Pain Point:**
1. **Frequency**: How often did this come up?
2. **Severity**: How much did it hurt?
3. **Workaround cost**: Time spent working around it?
4. **Generality**: Would others hit this?
5. **V1 solution exists?**: Can we import from archive?

### Expected Pain Point Categories

**Memory & Context**
- Symptoms: Agent forgets previous interactions, restarts context
- Severity: TBD based on usage
- Potential solution: Memory plugin (Layer 2)

**Discovery & Exploration**
- Symptoms: Agent can't find files, can't explore directories
- Severity: TBD based on usage
- Potential solution: Discovery plugin (Layer 2)

**Configuration Management**
- Symptoms: Hardcoded values, need different configs per agent
- Severity: TBD based on usage
- Potential solution: Config plugin (Layer 2)

**Resource Management**
- Symptoms: API rate limits, cost tracking, quotas
- Severity: TBD based on usage
- Potential solution: Resource plugin (Layer 2)

**Validation & Safety**
- Symptoms: Bad outputs, unsafe operations, need guardrails
- Severity: TBD based on usage
- Potential solution: Validation layer (Layer 3, import from V1)

### Deliverable
**Pain Point Priority Matrix**:
```
High Frequency + High Severity = Build first
High Frequency + Low Severity = Nice to have
Low Frequency + High Severity = Edge case, defer
Low Frequency + Low Severity = Ignore
```

## Phase 4: First Plugin Design (Day 7)

### Objective
Design and spec first plugin based on pain analysis

### Plugin Selection Criteria
1. Most painful point from Phase 3
2. Clear plugin boundary
3. Optional (doesn't break core)
4. Testable independently

### Plugin Design Template

**For Selected Plugin:**
```markdown
## [Plugin Name] Plugin

**Problem**: [Specific pain point from real usage]
**Solution**: [What the plugin provides]
**API**: [How agents use it]
**Dependencies**: [What it needs]
**V1 Import**: [Can we reuse V1 code?]

### Core Trait
- trait [PluginName]
- Key methods
- Integration points

### Builder Integration
- How to add to agent builder
- Configuration options
- Opt-in mechanism

### Success Criteria
- Solves the pain point
- Optional (core works without it)
- Clean API
- Minimal dependencies
```

### Acceptance Criteria
- [ ] Plugin selected based on data
- [ ] Design document created
- [ ] V1 code assessed for reuse
- [ ] Implementation plan ready

## Week 3 Preview: Plugin Implementation

**If Week 2 goes well:**
- Implement first plugin (based on Phase 4 design)
- Add to 2-3 existing agents
- Measure impact (time saved, quality improved)
- Repeat pain analysis for second plugin

**Emergent possibilities:**
- Memory plugin if agents forget context
- Discovery plugin if exploration painful
- Config plugin if hardcoding frustrating
- Tool registry if tool management messy

## Risks & Mitigation

### Risk: Real Provider Integration Fails
**Likelihood**: Low (V1 providers proven)
**Impact**: High (blocks real usage)
**Mitigation**:
- Use V1 providers as fallback
- Start with most stable (OpenAI)
- Have mock as safety net

### Risk: No Compelling Use Cases
**Likelihood**: Medium (depends on creativity)
**Impact**: High (can't validate design)
**Mitigation**:
- Pick agents we actually need
- Start with file processor (universal need)
- Ask: "What would make our lives easier?"

### Risk: Sophistication Creep
**Likelihood**: High (natural tendency)
**Impact**: High (defeats minimal-first)
**Mitigation**:
- CLAUDE.md guards against it
- Pain-driven only rule
- Review decisions against "is this minimal?"

### Risk: Import Too Much from V1
**Likelihood**: Medium (V1 code is there)
**Impact**: Medium (complexity creeps back)
**Mitigation**:
- Only import specific files needed
- Simplify V1 code during import
- Prefer external crates first

## Success Metrics

### Quantitative
- **2+ agents built** for real tasks
- **Pain points documented** with frequency/severity data
- **1 plugin designed** based on real usage
- **Real LLM calls** working (not mock)

### Qualitative
- Agents **actually used** (not just demos)
- Team **wants to build more** agents
- Pain points are **specific** (not vague "needs X")
- Plugin design is **minimal** (not comprehensive)

## Decision Points

### Day 2: Provider Integration Approach
- Go/no-go on async-openai
- If no, import from V1 or build minimal

### Day 5: Plugin Priority
- Which pain point to solve first
- Build plugin now or wait for more data

### Day 7: Week 3 Scope
- Continue with plugin layer or
- Build more agents for more data

## Anti-Patterns to Avoid

### ❌ DON'T
- Build plugins before pain is felt
- Import entire V1 provider system
- Add features "because we might need them"
- Skip documentation of pain points
- Rush to sophisticated without validation

### ✅ DO
- Let real usage drive features
- Import minimal needed from V1
- Add features only when painful without them
- Document every pain point with specifics
- Stay disciplined on minimal-first

## Resources Needed

### Technical
- OpenAI or Anthropic API key
- Time for 2-3 agent builds (~2-4 hours each)
- Access to V1 archive code

### Human
- Developer time: ~20-30 hours total
- Real tasks to build agents for
- Discipline to resist sophistication creep

## Relationships
- **Follows:** [planning/v2-week-1-retrospective.md] - Week 1 completion
- **Implements:** [decisions/v2_strategic_reset.md] - Week 2-3 plan
- **Informs:** [planning/v2-week-3-plan.md] - Next sprint (TBD)
- **Guided by:** [foundation/project_definition.md] - Layer 2 objectives

## Metadata
- **Sprint:** V2 Week 2
- **Date Range:** October 13-19, 2025 (estimated)
- **Status:** PLANNED (Week 1 just completed)
- **Created:** October 13, 2025
- **Last Updated:** October 13, 2025
- **Next Review:** End of Week 2 (October 19, 2025)

## Appendix: Quick Reference

### Week 2 Phases Summary
1. **Days 1-2**: Real provider integration
2. **Days 3-5**: Build 2 real agents
3. **Day 6**: Analyze pain points
4. **Day 7**: Design first plugin

### Key Questions to Answer
- Which provider? (OpenAI, Anthropic, or import V1?)
- Which agents to build? (File processor, git helper, doc generator?)
- Which pain point hurts most? (Memory, discovery, config, resources?)
- Which plugin first? (Based on pain data)

### Week 2 Completion Criteria
✅ Real LLM integrated
✅ 2+ agents built and used
✅ Pain points documented
✅ First plugin designed
✅ Ready for Week 3 implementation

---

**Status**: Ready to begin Week 2 - Real usage validation phase
