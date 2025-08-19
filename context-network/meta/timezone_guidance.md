# Timezone and Date Handling Guidance

## Purpose
Establish clear guidelines for handling dates and timezones in context network documentation to prevent incorrect timestamps and maintain consistency.

## Classification
- **Domain:** Meta
- **Stability:** Static
- **Abstraction:** Operational
- **Confidence:** Established

## The Problem
AI models may use training cutoff assumptions for dates instead of actual system time, and may default to UTC instead of user's local timezone, leading to incorrect timestamps in documentation.

## Solution: Always Use System Calls

### Required Commands
```bash
# Get current system time (UTC)
date

# Get user's local time (US Central timezone)
TZ='America/Chicago' date
```

### Rules for Documentation

1. **NEVER hardcode dates** based on model assumptions
2. **ALWAYS query system time** using bash commands
3. **RESPECT user timezone** - use `TZ='America/Chicago' date` for this project
4. **UPDATE timestamps** when modifying existing documents
5. **MARK backdated timestamps** clearly with "(estimated)" when needed

### Implementation Standards

#### When Creating New Documents
- Use `TZ='America/Chicago' date` to get current local time
- Format as YYYY-MM-DD for consistency
- Include in metadata sections

#### When Updating Existing Documents
- Update "Last Updated" timestamp to current local time
- Preserve original creation date unless correcting an error
- Mark corrected dates with "(corrected from incorrect assumption)"

#### When Backdating for Context
- Clearly mark estimated dates: "2025-01-17 (estimated)"
- Explain rationale: "Based on git history/context clues"
- Distinguish from actual system-queried dates

### Example Corrections Made
On August 18, 2025, corrected multiple documents that had:
- **Incorrect**: "2025-01-19" (model assumption)
- **Incorrect**: "2025-08-19" (UTC instead of local time)
- **Correct**: "2025-08-18" (actual US Central date via system call)

### Quality Assurance
- CI/CD pipeline caught date inconsistencies during code review
- Human reviewer identified the timezone/date assumption problem
- All documentation updated to use correct local time

## Implementation in CLAUDE.md
This guidance has been added to CLAUDE.md under "Timezone and Date Requirements" section with:
- Clear prohibition against hardcoded dates
- Required bash commands for date retrieval
- Examples and enforcement in quality checklist

## Enforcement
- Code review process checks for hardcoded dates
- Documentation updates must include correct timestamps
- AI assistants must use system calls for all date operations

## Relationships
- **Implements:** General documentation quality standards
- **Supports:** Context network maintenance procedures
- **Referenced by:** CLAUDE.md timezone requirements

## Metadata
- **Created:** August 18, 2025
- **Last Updated:** August 18, 2025
- **Updated By:** Development Team
- **Priority:** HIGH - Documentation Quality

## Change History
- August 18, 2025: Created timezone guidance after date correction incident