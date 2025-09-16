# Process: Code Review Triage Standards

## Classification
- **Domain**: Process / Quality Assurance
- **Stability**: Semi-stable
- **Abstraction**: Procedural
- **Confidence**: Established (validated through provider testing utilities code review)

## Overview

Standard practice for intelligently handling multiple code review recommendations using risk-based triage. This ensures continuous improvement while maintaining system stability.

## Core Standard: Risk-Effort Decision Matrix

### Standard Triage Categories

**APPLY IMMEDIATELY** - All conditions must be met:
- **Effort**: Trivial (< 5 min) OR Small (5-30 min)
- **Risk**: Low (style, docs, isolated) OR Medium with excellent test coverage
- **Dependencies**: Independent OR Local context only
- **Clarity**: Fix is obvious and unambiguous
- **Safety**: Cannot break existing functionality

**DEFER TO PLANNED TASK** - Any condition triggers deferral:
- **Effort**: Large (> 60 min) OR Medium without clear scope
- **Risk**: High (architecture, external APIs, data handling)
- **Dependencies**: System-wide OR requires team discussion
- **Complexity**: Needs design decisions OR performance analysis
- **Scope**: Could introduce breaking changes
- **Business Logic**: Touches critical functionality

## Standard Assessment Process

### Step 1: Rapid Initial Categorization (5 minutes)
For each recommendation:
1. **Extract Action**: What specifically needs to be done?
2. **Identify Location**: Which files/systems affected?
3. **Assess Scope**: Single method, file, or system-wide?
4. **Note Context**: Any special considerations or constraints?

### Step 2: Apply Decision Matrix (10 minutes)
Rate each recommendation:

**Effort Scale:**
- **Trivial**: Single line, constant extraction, typo fix
- **Small**: Single method/file, simple refactor, obvious improvement
- **Medium**: 2-3 files, pattern standardization, requires understanding
- **Large**: Multiple files/systems, architectural change, research needed

**Risk Scale:**
- **Low**: Documentation, dead code, style consistency, obvious fixes
- **Medium**: Logic changes with tests, refactoring with validation
- **High**: API changes, data handling, security, performance, external integrations

### Step 3: Standard Application (Variable time)
**For Immediate Items:**
1. **Group Related Changes** - Apply similar fixes in single commit
2. **Test-First When Needed** - Add tests for logic changes
3. **Validate Immediately** - Run tests/linting after changes
4. **Document Rationale** - Brief note why applied immediately

**For Deferred Items:**
1. **Create Clear Task** - Actionable title and acceptance criteria
2. **Estimate Effort** - Realistic time estimate for planning
3. **Note Dependencies** - What's needed before work can begin
4. **Link to Source** - Reference to original review/recommendation

## Standard Templates

### Immediate Application Commit Message
```
fix: [brief description from review recommendation]

- Applied [reviewer name] recommendation from [PR/review link]
- Changes: [specific changes made]
- Risk: Low - [brief justification]
- Tests: [passed/added as needed]
```

### Deferred Task Template
```markdown
# [Clear Action-Oriented Title]

## Source
- **From**: [Code review/audit/source]
- **Reviewer**: [Name if applicable]
- **Original Context**: [Link to PR/review]

## Recommendation
[Exact quote or paraphrase of recommendation]

## Rationale for Deferral
- **Effort**: [Estimated time and complexity]
- **Risk**: [Potential impacts and risks]
- **Dependencies**: [What needs to happen first]
- **Complexity**: [Design/analysis needed]

## Acceptance Criteria
- [ ] [Specific outcome 1]
- [ ] [Specific outcome 2]
- [ ] [Testing/validation requirements]
- [ ] [Documentation updates needed]

## Success Metrics
[How to measure successful completion]

## Related Work
[Links to related tasks/decisions]
```

## Standard Quality Gates

### Before Applying Any Recommendation
- [ ] **Clear Understanding**: You can explain the change and its impact
- [ ] **Test Coverage**: Existing tests will catch regressions
- [ ] **Scope Boundary**: Change won't affect unrelated functionality
- [ ] **Rollback Plan**: Can easily revert if issues arise

### After Applying Immediate Changes
- [ ] **All Tests Pass**: No regressions introduced
- [ ] **Linting Clean**: Code style standards maintained
- [ ] **Behavior Preserved**: No functional changes unless intended
- [ ] **Documentation Current**: Comments/docs reflect changes

### For Deferred Tasks
- [ ] **Actionable Title**: Clear what needs to be done
- [ ] **Acceptance Criteria**: Specific, measurable outcomes
- [ ] **Realistic Estimate**: Time estimate based on actual complexity
- [ ] **Proper Category**: Placed in correct task category (bug/feature/debt)

## Standard Team Practices

### During Code Reviews
1. **Provide Context**: Explain why each recommendation matters
2. **Indicate Priority**: Suggest immediate vs future for each item
3. **Group Related Items**: Bundle similar recommendations together
4. **Estimate Impact**: Note effort/risk for complex recommendations

### During Review Response
1. **Acknowledge All Items**: Address every recommendation explicitly
2. **Apply Triage Standards**: Use consistent decision criteria
3. **Communicate Decisions**: Explain rationale for deferrals
4. **Link Created Tasks**: Reference tasks created for deferred items

### During Sprint Planning
1. **Review Deferred Tasks**: Regularly evaluate accumulated technical debt
2. **Balance Priorities**: Mix feature work with quality improvements
3. **Consider Dependencies**: Schedule prerequisite work appropriately
4. **Track Patterns**: Notice recurring issues that need systemic fixes

## Success Metrics

### Process Effectiveness
- **Triage Speed**: Average time to process recommendations < 15 minutes
- **Decision Consistency**: Same recommendations triaged similarly across team
- **Regression Rate**: Applied changes introduce zero breaking changes
- **Task Quality**: Deferred tasks have clear acceptance criteria

### Continuous Improvement
- **Pattern Recognition**: Identify systemic issues from repeated recommendations
- **Process Evolution**: Regular refinement of triage criteria based on experience
- **Knowledge Sharing**: Team learns from triage decisions and outcomes
- **Technical Debt Management**: Steady progress on deferred items

## Common Decision Examples

### Apply Immediately
- Extract magic numbers to constants
- Remove dead code with #[allow(dead_code)]
- Fix typos in comments/documentation
- Improve error messages for clarity
- Add missing error handling for simple cases

### Defer to Tasks
- Refactor to use different libraries/frameworks
- Change API interfaces or method signatures
- Implement new architectural patterns
- Add complex new functionality
- Performance optimizations requiring benchmarking

### Conditional Decisions
- **Simple Abstractions**: Apply if < 20 lines, defer if complex
- **Test Additions**: Apply for obvious missing cases, defer for complex scenarios
- **Documentation**: Apply for corrections, defer for comprehensive rewrites

## Integration Points

### With Development Workflow
- **Pre-commit**: Ensure changes pass all quality gates
- **CI/CD**: Automated validation of applied changes
- **Monitoring**: Track metrics on triage effectiveness

### With Team Practices
- **Retrospectives**: Review triage decisions and outcomes
- **Knowledge Sharing**: Document patterns and learnings
- **Onboarding**: Train new team members on triage standards

### With Quality Management
- **Technical Debt Tracking**: Monitor accumulation and resolution
- **Risk Assessment**: Regular evaluation of high-risk deferred items
- **Continuous Improvement**: Evolution of triage criteria

---

**Standard Adoption**: This process should be used consistently across all code reviews to ensure reliable quality improvement while maintaining system stability.

**Validation**: Successfully applied to 6 code review recommendations - 3 applied safely, 3 properly deferred with detailed tasks, zero regressions introduced.