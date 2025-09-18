# Code Review Recommendation Triage Process

## Process Overview
Systematic approach for handling code review recommendations by intelligently splitting them into immediate actions and future tasks based on effort, risk, and dependencies.

## Classification Matrix

### Apply Immediately When ALL Of:
- **Effort**: Trivial (< 5 min) or Small (5-30 min)
- **Risk**: Low (style, docs, isolated cleanup)
- **Dependencies**: Independent or Local context only
- **Clarity**: Fix is obvious and well-defined
- **Safety**: Won't break existing functionality

### Defer to Tasks When ANY Of:
- **Effort**: Large (> 60 min) or affects multiple systems
- **Risk**: High (architecture, external APIs, data handling)
- **Dependencies**: System-wide knowledge or team discussion needed
- **Uncertainty**: Requires design decisions or performance analysis
- **Breaking Changes**: Could affect existing functionality

## Triage Categories

### Security Issues → ALWAYS Apply Now
- Hardcoded secrets/credentials
- Injection vulnerabilities
- Exposed sensitive data
- Missing authentication/authorization

### Code Quality → Selective Application

**Apply Now**:
- Dead code removal
- Simple variable renames
- Missing error handling (simple cases)
- Magic number extraction
- Obvious duplications (< 10 lines)

**Defer to Tasks**:
- Large-scale refactoring
- Architecture changes
- Complex abstraction creation
- Cross-cutting concerns

### Test Improvements → Usually Apply Now
- Adding missing assertions
- Fixing test naming
- Simple test isolation fixes
- Adding constants for test data

**Defer When**:
- Complex test infrastructure changes
- Test framework modifications
- Performance test requirements

### Documentation → Apply Now
- Adding missing comments
- Fixing typos
- Clarifying confusing names
- Updating incorrect docs

## Process Steps

### 1. Assessment Phase
For each recommendation:
1. **Extract specifics**: File paths, line numbers, exact changes needed
2. **Assess effort**: Time required and files affected
3. **Evaluate risk**: Potential for breaking changes or regressions
4. **Check dependencies**: Required knowledge or team input
5. **Verify safety**: Can be applied without extensive testing

### 2. Decision Phase
Apply decision matrix:
```
if (effort <= Small AND risk <= Low AND dependencies <= Local AND clear_fix AND safe):
    apply_immediately()
else:
    defer_to_task()
```

### 3. Immediate Application
For "Apply Now" items:
1. **Validate**: Ensure fix is correct and complete
2. **Apply**: Make the change incrementally
3. **Test**: Run relevant tests to ensure no regressions
4. **Document**: Note what was changed and why

### 4. Task Creation
For deferred items:
1. **Create task file**: Use standard task template
2. **Include context**: Original recommendation and decision rationale
3. **Set priority**: Based on business impact and technical debt
4. **Define acceptance criteria**: Clear success conditions
5. **Estimate effort**: More detailed analysis for planning
6. **Link to source**: Reference to original review

### 5. Validation
After all decisions:
- [ ] All immediate fixes tested and validated
- [ ] All deferred items have tasks created
- [ ] No recommendations lost or forgotten
- [ ] Changes are documented and justified

## Task Creation Templates

### For Refactoring Tasks
```markdown
# Task: [Clear Action-Oriented Title]

## Original Recommendation
[Direct quote from review]

## Why Deferred
[Specific reasons - effort, risk, dependencies]

## Acceptance Criteria
- [ ] Specific measurable outcomes
- [ ] Test requirements
- [ ] Documentation updates

## Implementation Notes
- Files to modify
- Key considerations
- Potential challenges
```

### For Quality Improvements
```markdown
# Task: [Standard/Policy Title]

## Problem Statement
[What inconsistency or quality issue exists]

## Scope
[Which files/areas affected]

## Standards to Establish
[What guidelines to create]

## Success Metrics
[How to measure completion]
```

## Quality Guidelines

### Immediate Fixes
1. **Never break working code** - If uncertain, defer
2. **Maintain test coverage** - Run tests after each change
3. **Keep changes isolated** - One concern per fix
4. **Document decisions** - Note what and why changed

### Task Creation
1. **Clear actionable titles** - Specific and implementable
2. **Complete context** - Include original recommendation
3. **Realistic estimates** - Based on actual complexity
4. **Proper categorization** - Place in correct task domain

## Success Metrics

### Process Effectiveness
- **Quick Wins**: Count of immediate fixes applied safely
- **Risk Avoidance**: Complex changes properly deferred
- **Quality Impact**: Measurable improvement in code quality
- **No Regressions**: Zero issues introduced by immediate fixes

### Task Quality
- **Clear Requirements**: All deferred tasks have clear acceptance criteria
- **Proper Priority**: Tasks prioritized by business and technical impact
- **Complete Context**: Original recommendations preserved
- **Actionable Plans**: Tasks include specific implementation guidance

## Tools and Integration

### Review Tools
- Code review platforms (GitHub, GitLab, etc.)
- Static analysis tools (clippy, ESLint, etc.)
- Security scanners
- Documentation linters

### Task Management
- Context network task files
- Issue tracking systems
- Project management tools
- Sprint planning integration

### Automation Opportunities
- Automated triage for obvious cases (typos, formatting)
- Template generation for common task types
- Integration with CI/CD for immediate fixes
- Metrics collection and reporting

## Examples and Case Studies

### Successful Immediate Application
**Recommendation**: "Extract magic string 'Hello world!' to named constant"
**Decision**: Apply Now (Trivial effort, zero risk, clear improvement)
**Outcome**: Clean code with better maintainability, no regressions

### Proper Deferral
**Recommendation**: "Refactor authentication system to use JWT tokens"
**Decision**: Defer (High risk, system-wide impact, needs architecture review)
**Outcome**: Comprehensive task created with proper planning requirements

### Avoided Pitfall
**Recommendation**: "Extract duplicate validation code between providers"
**Initial Impulse**: Apply immediately (code duplication is bad)
**Correct Decision**: Defer (requires design decisions about shared utilities)
**Outcome**: Proper task created with architectural considerations

This process has proven effective for maintaining code quality while avoiding scope creep and ensuring systematic improvement over time.