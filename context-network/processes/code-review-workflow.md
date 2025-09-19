# Code Review Workflow

## Overview

Structured approach to code review that emphasizes quality, knowledge sharing, and continuous improvement.

## Review Process

### 1. Pre-Review Preparation

#### Author Responsibilities
- [ ] All tests pass locally
- [ ] Code follows project style guidelines
- [ ] Documentation updated for API changes
- [ ] Self-review completed with checklist
- [ ] PR description includes context and testing approach

#### Pre-Review Checklist
```markdown
## Change Summary
- **Type**: [Feature/Bug Fix/Refactor/Documentation]
- **Impact**: [High/Medium/Low]
- **Testing**: [Unit/Integration/Manual/Performance]

## Changes Made
- [ ] Describe what changed
- [ ] Explain why it changed
- [ ] List any breaking changes

## Testing Strategy
- [ ] New tests added
- [ ] Existing tests updated
- [ ] Manual testing completed
- [ ] Performance impact assessed
```

### 2. Review Assignment

#### Reviewer Selection
- **Domain Expert**: Someone familiar with the affected area
- **Fresh Eyes**: Someone less familiar for perspective
- **Security Review**: For changes affecting security boundaries
- **Performance Review**: For changes affecting critical paths

#### Review Types
- **Standard Review**: All code changes
- **Security Review**: Authentication, authorization, data handling
- **Architecture Review**: Major design changes
- **Performance Review**: Critical path or resource-intensive changes

### 3. Review Execution

#### Review Priorities
1. **Correctness**: Does the code do what it's supposed to do?
2. **Security**: Are there any security vulnerabilities?
3. **Performance**: Will this impact system performance?
4. **Maintainability**: Can this code be easily maintained?
5. **Style**: Does it follow project conventions?

#### Review Questions
- Is the code easy to understand?
- Are edge cases handled appropriately?
- Is error handling comprehensive?
- Are there any race conditions or concurrency issues?
- Is the test coverage adequate?
- Are there any security implications?

### 4. Feedback Guidelines

#### Providing Feedback
- **Be Specific**: Point to exact lines and suggest improvements
- **Be Constructive**: Explain the reasoning behind suggestions
- **Be Educational**: Share knowledge and best practices
- **Be Respectful**: Focus on the code, not the person

#### Feedback Categories
- **Must Fix**: Critical issues that block merge
- **Should Fix**: Important improvements that should be addressed
- **Consider**: Suggestions for improvement
- **Nitpick**: Minor style or preference issues

#### Example Feedback
```markdown
**Must Fix**: This SQL query is vulnerable to injection attacks.
Suggestion: Use parameterized queries instead.

**Should Fix**: This error handling is too generic.
Suggestion: Provide more specific error messages for debugging.

**Consider**: This function is getting complex.
Suggestion: Consider breaking it into smaller functions.

**Nitpick**: Missing newline at end of file.
```

### 5. Response and Resolution

#### Author Response
- Address all "Must Fix" items before requesting re-review
- Respond to feedback with explanations or questions
- Update code based on valid suggestions
- Thank reviewers for their time and insights

#### Reviewer Follow-up
- Re-review after changes are made
- Verify that concerns were addressed
- Approve when satisfied with changes

## Quality Gates

### Automated Checks
- [ ] All tests pass
- [ ] Code coverage meets threshold (90%+)
- [ ] Static analysis passes
- [ ] Security scan passes
- [ ] Performance benchmarks within limits

### Manual Review Requirements
- [ ] At least one technical review approval
- [ ] Security review for sensitive changes
- [ ] Architecture review for major changes
- [ ] Documentation review for user-facing changes

## Special Review Types

### Security Review Checklist
- [ ] Input validation and sanitization
- [ ] Authentication and authorization
- [ ] Data encryption and protection
- [ ] Secure communication protocols
- [ ] Error handling doesn't leak sensitive info
- [ ] Access controls are appropriate

### Performance Review Checklist
- [ ] Algorithm complexity analysis
- [ ] Memory usage assessment
- [ ] Database query optimization
- [ ] Caching strategy evaluation
- [ ] Load testing results
- [ ] Resource cleanup verification

### API Review Checklist
- [ ] API design follows REST principles
- [ ] Backwards compatibility maintained
- [ ] Error responses are consistent
- [ ] Rate limiting considerations
- [ ] Documentation is complete
- [ ] Versioning strategy is clear

## Metrics and Improvement

### Review Metrics
- **Review Time**: Average time from request to approval
- **Review Cycles**: Number of back-and-forth iterations
- **Defect Discovery**: Issues caught in review vs production
- **Knowledge Sharing**: Cross-team review participation

### Continuous Improvement
- Monthly review process retrospectives
- Reviewer training and calibration
- Automation of common feedback items
- Process refinement based on metrics

## Tools and Templates

### Review Tools
- GitHub/GitLab PR reviews
- Code analysis tools (SonarQube, CodeClimate)
- Security scanning (Dependabot, Snyk)
- Performance monitoring integration

### Templates
- [PR Description Template](pr-template.md)
- [Review Checklist Template](review-checklist.md)
- [Security Review Template](security-review-template.md)

## Related Documentation

- [Task Planning and Prioritization](task-planning-and-prioritization.md)
- [Risk Assessment Framework](risk-assessment-framework.md)
- [Error-Driven Development](../methodologies/error-driven-development.md)

## Success Criteria

- **Quality**: Defects caught in review, not production
- **Speed**: Reviews completed within 24 hours
- **Learning**: Knowledge sharing across team members
- **Consistency**: Uniform code quality across the codebase