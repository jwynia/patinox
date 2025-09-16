# Process: Code Review Recommendation Triage

## Classification
- **Domain**: Process / Quality Assurance
- **Stability**: Semi-stable
- **Abstraction**: Procedural
- **Confidence**: Established (validated through implementation)

## Overview

This process defines how to intelligently handle multiple code review recommendations by separating immediate low-risk improvements from complex changes requiring planning.

## Core Principle

**Smart Triage Over Bulk Application**: Not all recommendations should be applied immediately. Some need careful planning, testing, and design consideration.

## Triage Decision Matrix

### Apply Immediately If ALL Of:
- **Effort**: Trivial (< 5 min) or Small (5-30 min)
- **Risk**: Low (style, documentation, isolated cleanup)
- **Dependencies**: Independent or Local context only
- **Clarity**: Fix is obvious and unambiguous
- **Safety**: Won't break existing functionality

### Defer to Planned Task If ANY Of:
- **Effort**: Large (> 60 minutes)
- **Risk**: High (architecture, external APIs, data handling)
- **Dependencies**: System-wide or requires team discussion
- **Complexity**: Requires design decisions or performance analysis
- **Scope**: Could introduce breaking changes
- **Business Logic**: Touches critical functionality

## Assessment Categories

### Effort Classification
- **Trivial**: < 5 minutes, single line changes (constants, comments, typos)
- **Small**: 5-30 minutes, single file changes (simple refactoring, error messages)
- **Medium**: 30-60 minutes, 2-3 files (pattern standardization, small APIs)
- **Large**: > 60 minutes, multiple files/systems (architecture changes)

### Risk Classification
- **Low**: Style, documentation, isolated cleanup, dead code removal
- **Medium**: Logic changes with good test coverage, refactoring with tests
- **High**: Architecture changes, external APIs, data handling, security

### Dependency Classification
- **Independent**: Can be done in isolation without context
- **Local**: Requires understanding of immediate module/component
- **System**: Requires broader architectural knowledge
- **Team**: Needs discussion, approval, or coordination

## Process Steps

### Phase 1: Parse and Categorize
1. Extract all actionable recommendations with specific locations
2. Assess each recommendation for effort, risk, and dependencies
3. Note any provided rationale or context
4. Group related recommendations

### Phase 2: Apply Immediate Fixes
For items marked "Apply Now":

1. **Test-First Approach** (when modifying logic):
   - Write failing test for the issue if none exists
   - Apply the fix
   - Verify test passes
   - Run existing test suite for regressions

2. **Safe Refactoring** (when cleaning code):
   - Ensure test coverage exists
   - Make incremental changes
   - Run tests after each change
   - Keep changes isolated and focused

3. **Documentation/Style** (when non-functional):
   - Apply directly with confidence
   - Verify formatting/linting passes

### Phase 3: Create Planned Tasks
For items marked "Defer to Task":

1. **Create Task Entry** with:
   - Clear, actionable title derived from recommendation
   - Context from original review/recommendation
   - Concrete acceptance criteria
   - Realistic effort estimate
   - Dependencies and risks noted
   - Link to original source

2. **Categorize by Type**:
   - Bug fixes → `/tasks/bugs/`
   - Refactoring → `/tasks/refactoring/`
   - Features → `/tasks/features/`
   - Technical debt → `/tasks/tech-debt/`

### Phase 4: Documentation and Validation
1. **Document Applied Changes**:
   - What was changed and why
   - Test coverage added/verified
   - Any risks or follow-ups identified

2. **Validate Task Creation**:
   - All deferred items have clear acceptance criteria
   - Priorities are appropriate for impact and risk
   - Dependencies are documented
   - Tasks are in appropriate categories

## Proven Application Examples

### Example: Apply Now
```
Recommendation: "Extract magic number 1000 to named constant DEFAULT_MAX_TOKENS"
Assessment: Trivial effort + Low risk + Independent + Obvious fix = APPLY NOW
Result: 2-minute change, improved maintainability, zero risk
```

### Example: Defer to Task
```
Recommendation: "Replace manual JSON string formatting with serde_json for safety"
Assessment: Small effort + Medium risk + Local dependencies + Design choice = DEFER
Result: 45-minute planned task with proper testing and validation
```

### Example: Conditional Application
```
Recommendation: "Standardize builder pattern across MockHttpBuilder methods"
Assessment: Medium effort + Medium risk + API changes = DEFER
Result: Planned refactoring task with backward compatibility analysis
```

## Quality Guidelines

1. **Never Break Working Code**: When uncertain about impact, always defer
2. **Maintain Test Coverage**: Add tests for any logic changes
3. **Preserve Behavior**: Refactoring shouldn't change functionality
4. **Document Decisions**: Explain rationale for deferring items
5. **Group Related Changes**: Keep commits logical and focused
6. **Incremental Progress**: Many small safe improvements > one risky change

## Special Handling Rules

### Critical Security Issues → ALWAYS APPLY NOW
- Hardcoded secrets/credentials
- SQL injection vulnerabilities
- XSS vulnerabilities
- Exposed sensitive data
- Missing authentication checks

Exception: If fix is complex (>30 min), create URGENT task with temporary mitigation

### High Priority Bugs → USUALLY APPLY NOW
- Null reference errors
- Unhandled promise rejections
- Clear logic errors
- Memory leaks (if isolated fix)

Exception: If requires architectural changes, defer with high priority

### Performance Issues → USUALLY DEFER
- Only apply if improvement is measurable and obvious
- Defer if benchmarking or analysis needed
- Create task with specific performance criteria

## Success Metrics

### Quantitative Measures
- **Risk Mitigation**: Count of high-risk items properly deferred
- **Quick Wins**: Count of low-risk items applied immediately
- **Regression Prevention**: Zero breaking changes from applied fixes
- **Task Quality**: All deferred items have clear acceptance criteria

### Qualitative Indicators
- **Team Confidence**: Developers feel safe applying the process
- **Code Quality**: Steady improvement without disruption
- **Process Efficiency**: Less time spent on recommendation debates
- **Documentation Quality**: Clear rationale for all decisions

## Common Anti-Patterns to Avoid

### "Apply Everything" Anti-Pattern
- **Problem**: Treating all recommendations as equally urgent
- **Risk**: Breaking working code, introducing regressions
- **Solution**: Use triage matrix consistently

### "Defer Everything" Anti-Pattern
- **Problem**: Creating tasks for trivial changes
- **Risk**: Process overhead, delayed improvements
- **Solution**: Apply genuinely low-risk improvements immediately

### "No Testing" Anti-Pattern
- **Problem**: Applying changes without validation
- **Risk**: Silent regressions, behavior changes
- **Solution**: Always run tests, add tests for logic changes

## Integration with Development Workflow

### Code Review Integration
- Apply this process during code review cycles
- Document triage decisions in PR comments
- Link created tasks back to original review

### Sprint Planning Integration
- Prioritize deferred tasks based on impact and risk
- Group related refactoring tasks for efficiency
- Schedule complex items for design review

### Continuous Improvement
- Track which recommendations are commonly deferred
- Identify patterns suggesting process improvements
- Evolve triage criteria based on team experience

## Related Processes

- **Links to**: Code review workflows, task management, sprint planning
- **Depends on**: Established testing practices, clear task creation templates
- **Enables**: Confident code improvement, risk-managed development

---

**Key Success Factor**: The goal is sustainable progress through intelligent risk management. Many small, safe improvements consistently applied create more value than occasional large, risky changes.

**Validation**: Successfully applied to 6 code review recommendations during Provider Testing Utilities implementation - 3 applied immediately (zero issues), 3 deferred to well-planned tasks.