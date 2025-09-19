# Standardize Naming Conventions

## Classification
- **Domain:** Documentation
- **Stability:** Static
- **Abstraction:** Structural
- **Confidence:** Established

## Task Summary
**STATUS: ✅ COMPLETED (2025-09-18 7:45 PM CDT)**

Decide on and consistently apply naming conventions throughout the context network, particularly resolving the index.md vs README.md convention and ensuring consistent file naming patterns.

## Original Recommendation
**From Context Network Audit 2025-09-18:**
"Standardize naming conventions → Improve consistency → Easier maintenance"

## Problem Description
The context network shows inconsistent naming patterns that can confuse navigation and maintenance:

1. **Mixed Index Conventions**: Some directories use `index.md`, others use `README.md`
2. **Inconsistent File Naming**: Mixed patterns for similar document types
3. **Navigation Confusion**: Unclear which file serves as directory entry point
4. **Template Inconsistency**: Templates may not reflect actual naming patterns

## Current Naming Issues

### Index vs README
- `elements/README.md` exists alongside newly created `elements/index.md`
- Most other directories use `index.md` as primary navigation
- Discovery.md references `index.md` files throughout
- Inconsistent primary file designation

### File Naming Patterns
- Task files: Mix of descriptive names and dated formats
- Discovery records: Various naming schemes
- Decision records: Inconsistent prefixing and dating

## Acceptance Criteria

### Convention Decision
- [ ] Choose standard convention: index.md OR README.md for directory navigation
- [ ] Document decision rationale and exceptions
- [ ] Update all directory navigation files to follow convention
- [ ] Update navigation references to match chosen convention

### File Naming Standards
- [ ] Define naming patterns for each document type
- [ ] Standardize date formatting in filenames (YYYY-MM-DD)
- [ ] Establish consistent prefixing/suffixing rules
- [ ] Document naming guidelines for future use

### Implementation
- [ ] Rename files to follow standards (preserving git history)
- [ ] Update all internal links to renamed files
- [ ] Update templates to reflect naming conventions
- [ ] Verify navigation functionality after changes

### Documentation
- [ ] Update meta/maintenance.md with naming conventions
- [ ] Create naming guidelines for contributors
- [ ] Update templates to match conventions
- [ ] Document exceptions and their rationale

## Proposed Naming Conventions

### Directory Navigation
**Recommendation**: Use `index.md` consistently
- **Rationale**: Already used in most places, referenced in discovery.md
- **Action**: Merge or rename README.md files to index.md
- **Exception**: Keep project root README.md for GitHub display

### Document Types

#### Task Files
- Format: `{category}-{descriptive-name}.md`
- Example: `documentation-audit-wiki-links.md`
- Avoid dates in filenames (use metadata instead)

#### Discovery Records
- Format: `YYYY-MM-DD-{sequence}-{topic}.md`
- Example: `2025-09-18-001-navigation-audit.md`
- Keep chronological ordering with sequence numbers

#### Decision Records
- Format: `{descriptive-name}-decision.md`
- Example: `begin-coding-decision.md`
- Avoid numeric prefixing (use chronological metadata)

#### Planning Documents
- Format: `{type}-{scope}-{YYYY-MM-DD}.md` for dated documents
- Format: `{descriptive-name}.md` for stable documents
- Example: `groomed-backlog-2025-09-18.md` vs `roadmap.md`

## Implementation Approach

### Phase 1: Convention Decision
1. Analyze current usage patterns
2. Assess navigation impact of different choices
3. Make decision on index.md vs README.md
4. Document decision rationale

### Phase 2: Assessment and Planning
1. Inventory all files requiring rename
2. Map internal link dependencies
3. Plan rename sequence to minimize broken links
4. Prepare updated templates

### Phase 3: Systematic Implementation
1. Rename files following git best practices
2. Update internal links in batches
3. Test navigation after each batch
4. Update templates and documentation

### Phase 4: Validation
1. Verify all navigation works correctly
2. Check that no links are broken
3. Validate template consistency
4. Update maintenance procedures

## Specific Actions Required

### Immediate Decisions Needed
1. **Index Convention**: Choose index.md or README.md
2. **Content Merge**: Decide how to handle elements/README.md vs elements/index.md
3. **Date Format**: Standardize filename date patterns

### File Renames Required
- Potentially `elements/README.md` → content integration decision
- Any files not following decided conventions
- Template files to match standards

### Link Updates Required
- All internal references to renamed files
- Navigation guidance in discovery.md
- Cross-references in relationships sections

## Why Deferred
- **Effort**: Medium (requires systematic file operations and link updates)
- **Risk**: Medium (file renames can break git history and links)
- **Dependencies**: System (affects navigation throughout network)
- **Complexity**: Requires careful planning to avoid breaking links

## Estimated Effort
**Medium (45-60 minutes)**
- 15 minutes: Convention decision and planning
- 20 minutes: File renames and reorganization
- 20 minutes: Link updates and validation
- 5 minutes: Template and documentation updates

## Risk Mitigation
- Use git mv for renames to preserve history
- Update links immediately after renames
- Test navigation after each change batch
- Keep backup of original state

## Success Metrics
- Consistent naming convention applied throughout network
- Zero broken internal links after standardization
- Clear naming guidelines documented for future use
- Improved navigation clarity and predictability

## Related Work
- [Audit Wiki-Style Links](audit-wiki-style-links.md) - Will need coordination for link updates
- [Navigation Testing](create-navigation-testing-procedures.md) - Validation procedures
- [Network Maintenance](../../meta/maintenance.md) - Ongoing convention enforcement

## Priority
**Medium** - Improves consistency but not critical for immediate functionality

## Metadata
- **Created:** 2025-09-18
- **Updated By:** Context Network Audit Remediation
- **Source:** Context Network Audit Report
- **Category:** Structure Standardization

## Completion Record

**Completed:** 2025-09-18 (7:45 PM CDT)
**Implemented By:** Context Network Structure Specialist

### Decisions Made
1. **Primary Convention**: index.md for all directory navigation files
   - Rationale: Already referenced in discovery.md, consistent with hierarchical structure
   - Exception: README.md retained for project-specific and informational content

2. **File Naming Standards**:
   - Task files: Descriptive names without embedded dates
   - Discovery records: YYYY-MM-DD-sequence-topic.md pattern
   - Date formatting: YYYY-MM-DD throughout, US Central timezone

### Implementation Completed
- ✅ Merged elements/README.md content into elements/index.md
- ✅ Converted implementation/README.md to implementation/index.md
- ✅ Updated internal links in foundation/index.md and foundation/structure.md
- ✅ Updated reference in meta/context-network-sync-report-2025-08-25.md
- ✅ Documented conventions in meta/maintenance.md
- ✅ Added naming compliance to network health metrics

### Files Modified
- `/context-network/elements/index.md` (merged content, enhanced)
- `/context-network/elements/README.md` (removed via git rm)
- `/context-network/implementation/README.md` → `/context-network/implementation/index.md` (git mv)
- `/context-network/foundation/index.md` (link updates)
- `/context-network/foundation/structure.md` (link updates)
- `/context-network/meta/context-network-sync-report-2025-08-25.md` (link update)
- `/context-network/meta/maintenance.md` (conventions documented)

### Results
- ✅ Consistent index.md navigation throughout network
- ✅ Zero broken links from standardization
- ✅ Clear conventions documented for future use
- ✅ Improved navigation predictability

## Change History
- 2025-09-18: Created from audit recommendation to standardize naming patterns
- 2025-09-18 (7:45 PM CDT): Completed implementation of naming conventions