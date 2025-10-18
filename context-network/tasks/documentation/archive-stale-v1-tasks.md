# Archive Stale V1 Task Files

## Purpose
Clean up task directory by archiving V1-related tasks that are no longer relevant to V2 minimal-first approach.

## Classification
- **Domain:** Project Maintenance
- **Stability:** Static
- **Abstraction:** Operational
- **Confidence:** High

## Task Details

**Created**: 2025-10-17
**Source**: Context Network Audit (October 16, 2025)
**Priority**: High
**Effort**: Medium (3-4 hours)
**Type**: Maintenance / Cleanup

## Problem Statement

The `context-network/tasks/` directory contains 34 files >30 days old, many referencing V1 sophisticated-first work that was archived on October 12, 2025 during the V2 strategic reset. These outdated tasks create confusion about what work is active vs. obsolete.

## Examples of Stale V1 Tasks

- `task-005-openrouter-provider-2025-08-20.md` - V1 provider implementation
- `task-006-extract-streaming-validation-logic-2025-09-16.md` - V1 streaming work
- `task-007-implement-real-http-streaming-2025-09-16.md` - V1 streaming work
- Multiple LMStudio and Ollama provider tasks - V1 provider work
- Various streaming optimization and validation tasks - V1 sophisticated-first approach

## Acceptance Criteria

- [ ] Review all task files in `context-network/tasks/` from before October 12, 2025
- [ ] Categorize each task as:
  - **Archive to V1**: Relates to V1 sophisticated-first approach
  - **Keep active**: Still relevant to V2 minimal-first approach
  - **Update needed**: Relevant but needs V2 context update
- [ ] Move V1-related tasks to `context-network/archive/v1-research/tasks/`
- [ ] Update or archive tasks not relevant to V2
- [ ] Keep only V2-relevant tasks in active `context-network/tasks/`
- [ ] Document archival decisions in task file or archive README
- [ ] Verify no broken links from moving tasks

## Implementation Approach

### Step 1: Inventory (30 min)
```bash
find context-network/tasks -name "*.md" -type f ! -path "*/templates/*" -mtime +30 | sort > stale_tasks.txt
wc -l stale_tasks.txt  # Should show ~34 files
```

### Step 2: Categorization (1-2 hours)
For each file in stale_tasks.txt:
1. Read file header and content
2. Check if task references V1 concepts:
   - Sophisticated-first architecture
   - V1-specific providers (LMStudio, Ollama detailed implementations)
   - Streaming optimization (V1 focus area)
   - Validation pipelines (V1 Tower work)
   - Memory management utilities (V1 sophisticated features)
3. Categorize: Archive, Keep, or Update

### Step 3: Archive V1 Tasks (30-60 min)
```bash
mkdir -p context-network/archive/v1-research/tasks
# Move identified V1 tasks
mv context-network/tasks/V1_TASK.md context-network/archive/v1-research/tasks/
```

Create archive README:
```markdown
# V1 Archived Tasks

These tasks relate to the V1 sophisticated-first approach archived on October 12, 2025.

**Status**: Reference only - V1 work may be imported to V2 when validated through usage (Layer 4).

**See**: context-network/decisions/v2_strategic_reset.md for V2 strategy.
```

### Step 4: Update or Remove Other Stale Tasks (30-60 min)
- Update tasks still relevant to V2 with current context
- Archive tasks no longer relevant (not V1, just outdated)
- Document decisions in task metadata or archival notes

### Step 5: Verification (30 min)
- Search for broken links: `grep -r "tasks/.*2025-0[6-9]" context-network --include="*.md"`
- Update any references to moved tasks
- Verify active tasks directory only contains V2-relevant work

## Task Categories Likely to Archive

Based on V1/V2 split documented in strategic reset:

### Archive to V1 (High Confidence)
- Provider implementation tasks (LMStudio, Ollama detailed work)
- Streaming optimization tasks
- Validation pipeline tasks (Tower middleware)
- Memory management utility tasks
- Testing infrastructure guides (V1 comprehensive approach)

### Keep Active (V2 Relevant)
- Backlog structure tasks (still applicable)
- Documentation quality tasks (still relevant)
- Simple code quality tasks (universal)
- V2-specific plugin/layer work (created October 2025+)

### Requires Review
- Performance optimization tasks (may be V1-specific)
- Refactoring tasks (need V2 context check)
- Test improvement tasks (distinguish V1 comprehensive vs V2 minimal)

## Success Metrics

- Zero references to V1 concepts in active task directory
- Clear separation: V1 tasks in archive, V2 tasks in active
- No broken links from archival
- Active task count reduced from ~60 to ~25-30 (V2-relevant only)
- Future contributors can easily distinguish active vs. archived work

## Risks & Mitigation

**Risk**: Accidentally archive V2-relevant work
- **Mitigation**: Review each task carefully, when in doubt keep and update

**Risk**: Breaking links to archived tasks
- **Mitigation**: Search for references before moving, update or remove links

**Risk**: Losing V1 research value
- **Mitigation**: Archive to v1-research/ not delete, preserve for Layer 4 import

## Dependencies

- **Blocked by**: None (independent maintenance task)
- **Blocks**: None (improves clarity, doesn't block development)

## Related Work

- **Strategic reset decision**: `context-network/decisions/v2_strategic_reset.md`
- **V1 archival location**: `context-network/archive/v1-research/`
- **Context network audit**: Generated this task on October 17, 2025

## Metadata

- **Created**: 2025-10-17
- **Last Updated**: 2025-10-17
- **Created By**: Context Network Audit Remediation (Recommendation #4)
- **Priority**: High
- **Effort**: Medium (3-4 hours)
- **Risk**: Medium (could accidentally archive active work)

## Change History

- 2025-10-17: Created task to archive stale V1 tasks identified in context network audit
