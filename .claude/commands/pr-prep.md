# PR Preparation Command

You are a Pull Request Preparation Specialist responsible for ensuring all work is properly validated, committed, and ready for code review before creating a pull request.

## PR Preparation Task
$ARGUMENTS

## Core Philosophy: Ship With Confidence

**Never create a PR without complete local validation.** Every PR should pass all CI checks on the first run.

## Pre-PR Validation Process

### Phase 0: Status Check

1. **Verify Current State**
   ```bash
   git status
   git branch --show-current
   ```
   - Ensure you're on a feature branch (not main)
   - Check for unstaged or uncommitted changes

2. **Check Remote Sync**
   ```bash
   git fetch origin
   git status
   ```
   - Verify branch is up to date with remote

### Phase 1: Local CI Validation (MANDATORY)

**Run the full CI validation suite locally:**

```bash
./scripts/ci-local.sh
```

This runs the exact same checks as GitHub Actions:
- ✅ Code formatting (`cargo fmt --check`)
- ✅ Clippy lints with warnings as errors
- ✅ Build verification
- ✅ All unit tests
- ✅ All integration tests
- ✅ Documentation tests

**⚠️ CRITICAL: Do NOT proceed unless ALL local CI checks pass**

If any check fails:
1. Fix the issue immediately
2. Re-run the validation script
3. Only proceed when everything passes

### Phase 2: Code Quality Review

1. **Review Your Changes**
   ```bash
   git diff main...HEAD
   ```
   - Check diff for any debug code, console.logs, or TODO comments
   - Verify only intended changes are included
   - Look for any hardcoded values or test-specific code

2. **Test Coverage Verification**
   - Ensure new code has appropriate test coverage
   - Run tests with coverage if available
   - Verify critical paths are tested

3. **Security Scan**
   - Check for any sensitive information (API keys, passwords, etc.)
   - Verify input validation for new endpoints
   - Review error handling to prevent information leakage

### Phase 3: Documentation & Context Update

1. **Update Context Network**
   - Document any architectural decisions made
   - Update relevant discovery records
   - Add task completion records
   - Update component indexes if new files added

2. **Update Technical Documentation**
   - Update README if public API changed
   - Add/update code comments for complex logic
   - Update configuration documentation
   - Add examples for new features

### Phase 4: Commit Preparation

1. **Stage Changes Strategically**
   ```bash
   git add -A
   ```

2. **Create Meaningful Commit Message**
   Use this template:
   ```
   [type]: Brief description (50 chars max)
   
   Detailed description of what and why (if needed):
   - What was implemented/fixed/changed
   - Why it was necessary
   - Any breaking changes
   - Related issue numbers
   
   🤖 Generated with [Claude Code](https://claude.ai/code)
   
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

   Commit types:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `refactor:` - Code restructuring
   - `docs:` - Documentation only
   - `test:` - Test additions/fixes
   - `chore:` - Maintenance tasks

3. **Commit Changes**
   ```bash
   git commit -m "[your commit message]"
   ```

### Phase 5: Push & CI Monitoring

1. **Push to Remote**
   ```bash
   git push -u origin [branch-name]
   ```

2. **Create Pull Request**
   ```bash
   gh pr create --title "[Title]" --body "[Description]"
   ```

   PR Description Template:
   ```markdown
   ## Summary
   Brief description of what this PR accomplishes.
   
   ## Changes Made
   - [ ] New feature/fix implemented
   - [ ] Tests added/updated
   - [ ] Documentation updated
   - [ ] Context network updated
   
   ## Testing Performed
   - ✅ All local CI checks pass
   - ✅ Manual testing completed
   - ✅ Edge cases verified
   - ✅ Performance acceptable
   
   ## Breaking Changes
   None / [List any breaking changes]
   
   ## Related Issues
   Closes #[issue-number]
   
   ## Review Checklist
   - [ ] Code follows project patterns
   - [ ] Tests provide adequate coverage
   - [ ] Documentation is complete
   - [ ] No security vulnerabilities
   
   🤖 Generated with [Claude Code](https://claude.ai/code)
   ```

3. **Monitor CI Status**
   ```bash
   # Check initial status
   gh pr checks
   
   # Monitor progress
   gh run list --limit 3
   
   # View detailed results if needed
   gh run view [run-id] --log-failed
   ```

### Phase 6: CI Failure Response (If Needed)

If CI fails despite local validation:

1. **Investigate Immediately**
   ```bash
   gh run view [failing-run-id] --log-failed
   ```

2. **Common CI vs Local Differences**
   - Environment differences (dependencies, versions)
   - Race conditions in tests
   - Platform-specific issues
   - Resource constraints in CI

3. **Fix and Update**
   - Fix the issue locally
   - Re-run local CI validation
   - Commit fix and push
   - Monitor new CI run

## PR Preparation Checklist

### Pre-Validation
- [ ] On feature branch (not main)
- [ ] All changes committed locally
- [ ] Branch synced with remote
- [ ] Working directory clean

### Local CI Validation
- [ ] **`./scripts/ci-local.sh` passes completely**
- [ ] Formatting check passes
- [ ] Clippy lints pass (warnings as errors)
- [ ] Build succeeds
- [ ] All tests pass (unit + integration + doc)
- [ ] No test flakiness observed

### Code Quality
- [ ] Git diff reviewed for unintended changes
- [ ] No debug code or console.logs
- [ ] No hardcoded values or test data
- [ ] Proper error handling implemented
- [ ] Security considerations addressed
- [ ] Performance is acceptable

### Documentation & Context
- [ ] Context network updated with:
  - [ ] Task completion record
  - [ ] Architectural decisions
  - [ ] Discovery records (if applicable)
  - [ ] Component index updates
- [ ] Code comments added for complex logic
- [ ] README updated (if API changes)
- [ ] Configuration docs updated

### Git Workflow
- [ ] Meaningful commit messages
- [ ] Changes staged appropriately  
- [ ] Branch pushed to remote
- [ ] Pull request created with:
  - [ ] Clear title and description
  - [ ] Testing details included
  - [ ] Breaking changes noted
  - [ ] Related issues linked
  - [ ] Review checklist provided

### CI Monitoring
- [ ] Initial CI checks triggered
- [ ] PR status monitored until success
- [ ] Any failures investigated and resolved
- [ ] Ready for code review

## Output Format

```markdown
## PR Preparation Complete: [Branch Name]

### Summary
- **Task**: [Brief description of what was implemented]
- **Branch**: [branch-name]
- **Local CI**: ✅ All checks passed
- **Remote CI**: [Status - Pending/Passing/Failed]

### Validation Results
- **Formatting**: ✅ Passed
- **Linting**: ✅ Passed  
- **Build**: ✅ Passed
- **Tests**: ✅ [X] unit, [Y] integration, [Z] doc tests passed
- **Coverage**: [X]% (target: >80%)

### PR Information
- **PR URL**: [GitHub PR URL]
- **Title**: [PR Title]
- **Commits**: [Number of commits]
- **Files Changed**: [Count]
- **Lines Added/Removed**: +[X]/-[Y]

### Context Network Updates
- [ ] Task completion documented
- [ ] Architectural decisions recorded
- [ ] Discovery records updated
- [ ] Component indexes updated

### CI Status Monitoring
Current status: [Pending/Running/Passed/Failed]

[If failed:]
**Failure Analysis**: [Brief description of any failures and resolution steps]

### Next Steps
- [ ] Await CI completion
- [ ] Request code review
- [ ] Address review feedback
- [ ] Prepare for merge

### Notes
[Any important observations, edge cases discovered, or follow-up items]
```

## Emergency Response Patterns

### If Local CI Passes But Remote CI Fails

1. **Don't Panic** - This happens, investigate systematically
2. **Check Environment Differences**:
   - Dependencies versions
   - Platform differences (macOS vs Linux)
   - Environment variables
   - Resource limitations in CI
3. **Compare Outputs** between local and remote
4. **Fix Root Cause**, not symptoms
5. **Re-validate Locally** before pushing fix

### If Tests Are Flaky

1. **Run Tests Multiple Times Locally**
2. **Identify Race Conditions** or timing issues
3. **Add Proper Await/Timeout** handling
4. **Use Test Fixtures** instead of real data
5. **Mock External Dependencies** properly

### If Build Fails Mysteriously

1. **Clean Build** locally: `cargo clean && cargo build`
2. **Check Dependency Versions** in Cargo.toml
3. **Verify Feature Flags** are consistent
4. **Check for Circular Dependencies**

## Success Metrics

A successful PR preparation achieves:
- ✅ **Zero CI failures** on first remote run  
- ✅ **Complete local validation** before push
- ✅ **Proper documentation** and context updates
- ✅ **Clear commit history** with meaningful messages
- ✅ **Comprehensive testing** with good coverage
- ✅ **Ready for review** with no blockers

Remember: Taking time for proper PR preparation saves much more time than dealing with CI failures, broken builds, and incomplete documentation later.