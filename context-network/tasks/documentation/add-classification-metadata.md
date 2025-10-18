# Add Classification Metadata to Active Files

## Purpose
Systematically add classification metadata to active context network files that lack it, improving filtering, search, and understanding of content maturity.

## Classification
- **Domain:** Documentation Quality
- **Stability:** Static
- **Abstraction:** Operational
- **Confidence:** High

## Task Details

**Created**: 2025-10-17
**Source**: Context Network Audit (October 16, 2025) - Recommendation #6
**Priority**: Medium
**Effort**: Large (4-5 hours for 93 files)
**Type**: Quality Improvement / Batch Operation

## Problem Statement

Only 103/196 active files (53%) have classification metadata. The remaining 93 files (47%) lack formal classification, making it difficult to assess:
- **Stability**: How often does this change?
- **Confidence**: How reliable is this information?
- **Domain**: What knowledge area does this cover?
- **Abstraction**: What detail level is this?

## Classification System

Files should include this metadata:

```markdown
## Classification
- **Domain:** [Primary knowledge area]
- **Stability:** [Static/Semi-stable/Dynamic]
- **Abstraction:** [Conceptual/Structural/Detailed]
- **Confidence:** [Established/Evolving/Speculative]
```

### Classification Dimensions

**Domain** (Primary knowledge area):
- Project Foundation, Planning, Implementation, Quality, Process, Structure, etc.

**Stability** (Change frequency):
- **Static**: Fundamental principles unlikely to change
- **Semi-stable**: Established patterns that evolve gradually
- **Dynamic**: Frequently changing information

**Abstraction** (Detail level):
- **Conceptual**: High-level ideas and principles
- **Structural**: Organizational patterns and frameworks
- **Detailed**: Specific implementations and examples

**Confidence** (Information reliability):
- **Established**: Verified and reliable information
- **Evolving**: Partially validated but subject to refinement
- **Speculative**: Exploratory ideas requiring validation

## Acceptance Criteria

- [ ] Document which file types REQUIRE classification vs. OPTIONAL
- [ ] Identify the 93 files without classification metadata
- [ ] Categorize files by type (decision, planning, task, record, etc.)
- [ ] Add appropriate classification to each file
- [ ] Update `meta/maintenance.md` with classification standards
- [ ] Classification coverage increases from 53% to 80%+ for relevant file types
- [ ] Templates updated to include classification by default

## Implementation Approach

### Step 1: Define Classification Requirements (1 hour)

**Files that REQUIRE classification**:
- Decision records (`decisions/*.md`)
- Planning documents (`planning/*.md`)
- Research documents (`research/*.md`)
- Methodology documents (`methodologies/*.md`)
- Major process documents (`processes/*.md`)
- Foundation documents (`foundation/*.md`)
- Element/component docs (`elements/*.md`)
- Connection docs (`connections/*.md`)

**Files where classification is OPTIONAL**:
- Task files (transient by nature)
- Completion records (historical, metadata in content)
- Index files (organizational, not content)
- Templates (meta documents)
- README files (explanatory only)

### Step 2: Inventory Files Needing Classification (30 min)
```bash
# Find files without classification in required areas
for dir in decisions planning research methodologies processes foundation elements connections; do
  echo "=== $dir ==="
  grep -L "## Classification" context-network/$dir/*.md 2>/dev/null || echo "None missing"
done
```

### Step 3: Batch Add Classification (2-3 hours)

Group files by similarity and add appropriate classification:

**Example: Decision Records**
```markdown
## Classification
- **Domain:** Project Governance / Architecture / Process
- **Stability:** Static
- **Abstraction:** Policy / Structural / Detailed
- **Confidence:** Established
```

**Example: Planning Documents**
```markdown
## Classification
- **Domain:** Planning
- **Stability:** Dynamic
- **Abstraction:** Structural
- **Confidence:** Evolving
```

**Example: Process Documents**
```markdown
## Classification
- **Domain:** Project Operations
- **Stability:** Semi-stable
- **Abstraction:** Structural
- **Confidence:** Established
```

### Step 4: Update Standards Documentation (30 min)

Add to `meta/maintenance.md`:
```markdown
## Classification Requirements

### When Classification is Required
- Decision records - REQUIRED
- Planning documents - REQUIRED
- Research/methodology - REQUIRED
- Foundation documents - REQUIRED
- Major process documents - REQUIRED

### When Classification is Optional
- Task files (transient)
- Completion records (historical)
- Index files (organizational)
- Templates (meta)
- README files (explanatory)

### Classification Template
[Include standard classification block]
```

### Step 5: Update Templates (30 min)

Ensure all templates in `meta/templates/` include classification block with appropriate defaults.

### Step 6: Verification (30 min)
```bash
# Check classification coverage improved
# Required areas should have 90%+ coverage
for dir in decisions planning processes foundation; do
  total=$(find context-network/$dir -name "*.md" | wc -l)
  classified=$(grep -l "## Classification" context-network/$dir/*.md 2>/dev/null | wc -l)
  echo "$dir: $classified/$total classified"
done
```

## Classification Guidelines

### For Strategic Documents (Decisions, Planning)
- **Domain**: Usually "Project Governance" or "Planning"
- **Stability**: Mostly "Static" (decisions) or "Dynamic" (plans)
- **Abstraction**: Usually "Structural" or "Policy"
- **Confidence**: "Established" for approved decisions, "Evolving" for active planning

### For Process Documents
- **Domain**: "Project Operations"
- **Stability**: "Semi-stable" (processes evolve gradually)
- **Abstraction**: "Structural"
- **Confidence**: "Established" for proven processes, "Evolving" for new ones

### For Technical Documents (Elements, Connections)
- **Domain**: "Structure" or specific technical area
- **Stability**: "Semi-stable" (architecture evolves)
- **Abstraction**: "Structural" or "Detailed"
- **Confidence**: "Established" for built components, "Evolving" for planned

## Success Metrics

- Classification coverage: 53% → 80%+ for required file types
- Classification standards documented in meta/maintenance.md
- All templates include classification
- Filtering by stability/confidence becomes reliable
- Future files created with classification by default

## Risks & Mitigation

**Risk**: Inconsistent classification across similar files
- **Mitigation**: Define clear examples for each file type, batch similar files

**Risk**: Wrong classification assigned
- **Mitigation**: Classifications can be updated, not permanent; start conservative

**Risk**: Time-consuming for large batch
- **Mitigation**: Focus on high-value files first (decisions, planning), defer tasks/records

## Dependencies

- **Blocked by**: None
- **Blocks**: Better filtering and search (nice-to-have, not blocking)

## Related Work

- **Classification system**: Defined in `context-network/discovery.md`
- **Context network audit**: Generated this task October 17, 2025

## Metadata

- **Created**: 2025-10-17
- **Last Updated**: 2025-10-17
- **Created By**: Context Network Audit Remediation (Recommendation #6)
- **Priority**: Medium
- **Effort**: Large (4-5 hours for 93 files, but can be done incrementally)
- **Risk**: Low (metadata enhancement only)

## Change History

- 2025-10-17: Created task to systematically add classification metadata to improve content organization
