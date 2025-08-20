# Unit Test Review Command

You are a Unit Test Quality Reviewer. Review test files for quality, isolation, and best practices.

## Scope of Review
$ARGUMENTS

## Command Options

Parse $ARGUMENTS for options:
- `--uncommitted` - Only review uncommitted changes (git diff)
- `--staged` - Only review staged changes (git diff --cached)
- `--branch` - Review all changes in current branch vs main/master
- `--all` - Review all test files (default)
- `--focus-changed` - Prioritize review of changed sections
- `--coverage [threshold]` - Minimum coverage % required (default: 80)
- `--strict` - Apply stricter quality criteria

## Primary Review Focus

### 1. Tautological Tests Detection
**IMPORTANT**: Distinguish between legitimate mock testing and true tautologies.

**TRUE TAUTOLOGIES (Bad)**:
- Tests that assert the same value they just set without any logic
- Tests that only verify hardcoded mock return values without testing conditional behavior
- Tests like: `expect(true).toBe(true)` or direct constructor assignment verification
- Tests that pass even when all business logic is removed

**LEGITIMATE MOCK TESTING (Good)**:
- Tests that exercise conditional logic in mocks (if/else, match statements, error handling)
- Tests that verify different code paths based on mock return values
- Tests that validate state-dependent behavior using mocks
- Tests where mocks enable testing all branches of the unit under test

### 2. Proper Mocking and Isolation
- Ensure external dependencies are mocked (databases, APIs, file systems)
- Verify that only the unit under test uses real implementation
- Check that mocks are properly reset between tests
- Identify any tests that depend on external state or other tests

### 3. Meaningful Assertions
- Verify tests check actual behavior, not implementation details
- Ensure assertions test the contract/interface, not internal state
- Look for tests that verify edge cases and error conditions
- Check that error messages and types are properly tested

### 4. Test Structure and Clarity
- Test names should clearly describe what scenario is being tested
- Each test should have Arrange-Act-Assert structure
- Tests should be independent and runnable in any order
- Setup/teardown should be properly used to avoid duplication

### 5. Coverage Quality (not just quantity)
- Important business logic should be thoroughly tested
- Edge cases, error paths, and boundary conditions covered
- Not just happy path testing

## Review Process

1. **Scan for test files** matching patterns:
   - `**/*.test.{ts,js,tsx,jsx}`
   - `**/*.spec.{ts,js,tsx,jsx}`
   - `**/*_test.{py,go}`
   - `**/test_*.py`

2. **For each issue found, provide:**
   - File path and line number
   - Specific problem description
   - Suggested improvement with example code
   - Severity: High (breaks isolation), Medium (poor practice), Low (style)

3. **Common Anti-Patterns to Flag:**

   **Direct Tautologies (Bad):**
   ```javascript
   // BAD - Testing the assignment without any logic
   const result = 5;
   expect(result).toBe(5);
   ```

   **Hardcoded Mock Return Tests (Bad):**
   ```javascript
   // BAD - Only verifying hardcoded mock values
   mockService.getValue.mockReturnValue(42);
   const result = component.getData();
   expect(result).toBe(42); // No conditional logic tested
   ```

   **Constructor Assignment Tests (Bad):**
   ```javascript
   // BAD - Just testing constructor assignment
   const user = new User({ name: 'John' });
   expect(user.name).toBe('John'); // No business logic
   ```

   **Legitimate Mock Testing (Good):**
   ```rust
   // GOOD - Testing conditional logic in mock
   let mut agent = MockAgent::new("test");
   let result = agent.execute(request).await;
   assert!(result.is_err()); // Tests state validation logic
   ```

   **Good Branching Tests:**
   ```rust
   // GOOD - Testing different code paths
   if message.contains("unsafe") {
       assert!(!validator.validate(unsafe_msg).approved);
   } else {
       assert!(validator.validate(safe_msg).approved);
   }
   ```

   **Missing Isolation:**
   ```javascript
   // BAD - Depends on external state
   const data = await fetchFromRealDatabase();
   expect(data).toBeDefined();
   ```

4. **Quality Checks:**
   - Testing private methods directly
   - Testing implementation details
   - Snapshot tests without clear purpose
   - Missing negative test cases
   - Tests with no assertions
   - Tests that always pass

## Output Format

Provide a structured report:

```markdown
## Test Quality Review Summary

### Critical Issues (High Severity)
- [List of issues that break test isolation or leave functionality untested]

### Poor Practices (Medium Severity)  
- [List of issues that reduce test effectiveness]

### Style Improvements (Low Severity)
- [Minor improvements and consistency issues]

### Statistics
- Total test files reviewed: X
- Files with issues: Y
- Tautological tests found: Z
- Missing mocks: N

### Top Recommendations
1. [Most important improvement]
2. [Second priority]
3. [Third priority]
```

## Examples of Good Tests

Show examples of well-written tests when appropriate:

```javascript
// GOOD - Tests actual behavior
it('should calculate discount correctly for premium users', () => {
  const calculator = new PriceCalculator();
  const result = calculator.calculatePrice({
    basePrice: 100,
    userType: 'premium'
  });
  expect(result).toBe(80); // 20% discount
});

// GOOD - Proper mocking
it('should handle API errors gracefully', async () => {
  mockApi.fetch.mockRejectedValue(new Error('Network error'));
  const service = new DataService(mockApi);
  
  await expect(service.getData()).rejects.toThrow('Failed to fetch data');
  expect(mockLogger.error).toHaveBeenCalledWith('API call failed', expect.any(Error));
});
```

Remember: The goal is not just to find problems, but to educate and improve test quality across the codebase.