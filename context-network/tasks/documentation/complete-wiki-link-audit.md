# Complete Wiki-Link Audit and Conversion

## Purpose
Systematically audit all `[[wiki-style links]]` in the context network and convert them to standard markdown format.

## Classification
- **Domain:** Documentation Quality
- **Stability:** Static
- **Abstraction:** Operational
- **Confidence:** High

## Task Details

**Created**: 2025-10-17
**Source**: Context Network Audit (October 16, 2025) - Recommendation #5
**Priority**: Medium
**Effort**: Medium (2-3 hours)
**Type**: Technical Debt / Cleanup

## Problem Statement

Historical use of `[[wiki-style links]]` exists in the context network. Modern files use standard markdown `[text](path)` format, but legacy wiki-links may cause broken navigation.

An audit task file exists (`context-network/tasks/documentation/audit-wiki-style-links.md`) but hasn't been executed.

## Known Wiki-Link Examples

From existing audit task file:
- `[[tower-validation-pipeline-implementation]]`
- `[[validation-tdd-methodology]]`
- `[[validator-sorting-optimization]]`
- `[[html-sanitization-upgrade]]`
- `[[groomed-backlog-2025-09-15]]`
- `[[Async Testing Best Practices]]`
- `[[Error-Driven Development]]`
- `[[Code Review Workflow]]`
- `[[Task Planning and Prioritization]]`
- `[[Risk Assessment Framework]]`

## Acceptance Criteria

- [ ] Complete scan of all .md files for `[[wiki-links]]`
- [ ] Document all found wiki-links with:
  - Source file location
  - Target document (if exists)
  - Conversion needed or link removal
- [ ] Convert wiki-links to standard markdown format:
  - `[[target]]` → `[Target](path/to/target.md)`
- [ ] Create missing target documents OR remove invalid links
- [ ] Verify all converted links navigate correctly
- [ ] Zero `[[` patterns remain in active context network
- [ ] Update or archive the original audit task file

## Implementation Approach

### Step 1: Scan for Wiki-Links (30 min)
```bash
# Find all wiki-style links
grep -r "\[\[" context-network --include="*.md" ! -path "*/archive/*" > wiki_links_found.txt

# Count occurrences
wc -l wiki_links_found.txt
```

### Step 2: Categorize Links (1 hour)
For each wiki-link found:
1. Identify target document name
2. Check if target exists in context network
3. Categorize as:
   - **Convert**: Target exists, update to `[text](path)`
   - **Create**: Target should exist, create it then convert
   - **Remove**: Link is obsolete, remove it

### Step 3: Convert Links (30-60 min)
```bash
# For links with existing targets
# Example: [[Code Review Workflow]] in file X
# Find target: context-network/processes/code-review-workflow.md
# Convert to: [Code Review Workflow](../../processes/code-review-workflow.md)
```

Use relative paths from source file to target.

### Step 4: Create Missing Targets (30-60 min)
For high-value missing targets:
- `[[validation-tdd-methodology]]` → Create in processes/ or methodologies/
- `[[Async Testing Best Practices]]` → Create in processes/ or archive if V1-specific
- `[[Error-Driven Development]]` → Create in methodologies/ or reference existing

For low-value/obsolete targets:
- Remove the wiki-link
- Replace with plain text or remove sentence

### Step 5: Verification (30 min)
```bash
# Verify no wiki-links remain
grep -r "\[\[" context-network --include="*.md" ! -path "*/archive/*"
# Should return no results

# Spot-check converted links work
# Open several converted markdown links in editor/viewer
```

## Decision Matrix for Missing Targets

| Target Type | Action |
|-------------|--------|
| Process document | Create if generally useful, remove if V1-specific |
| Methodology | Create if validated pattern, remove if speculative |
| Task/planning doc | Check if exists under different name, otherwise remove |
| Decision record | Create if decision was made, remove if never decided |
| V1-specific | Remove link, add note "See V1 archive" if relevant |

## Special Handling

### V1-Specific Wiki-Links
- Don't create V1-specific targets in active network
- Convert to references to archive:
  - `[[tower-validation-pipeline]]` → See `archive/v1-research/...`

### Obsolete Grooming Backlogs
- `[[groomed-backlog-2025-09-15]]` → Archive or remove
- Current backlogs use by-status organization

## Success Metrics

- Zero `[[wiki-links]]` in active context network (excluding archive)
- All converted links use correct relative paths
- High-value missing targets created with proper classification
- Low-value links removed cleanly
- Navigation integrity maintained

## Dependencies

- **Blocked by**: None
- **Blocks**: None (improves navigation quality)

## Related Work

- **Original audit task**: `context-network/tasks/documentation/audit-wiki-style-links.md`
- **Context network audit**: Generated this updated task on October 17, 2025

## Metadata

- **Created**: 2025-10-17
- **Last Updated**: 2025-10-17
- **Created By**: Context Network Audit Remediation (Recommendation #5)
- **Priority**: Medium
- **Effort**: Medium (2-3 hours)
- **Risk**: Low (documentation cleanup)

## Change History

- 2025-10-17: Created comprehensive task based on existing audit task file and context network audit findings
