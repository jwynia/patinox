# In Progress

Tasks currently being actively worked on by team members.

## What Makes a Task "In Progress"?

A task is in progress when:
- ✅ Someone has started implementation
- ✅ Feature branch exists
- ✅ Commits have been made
- ⏳ Work is ongoing
- ⏳ Not yet ready for review

## Transition Rules

- **Ready** → **In Progress**: When `/implement [TASK-ID]` is run
- **In Progress** → **Completed**: When PR is merged or work is done
- **In Progress** → **Ready**: If work is paused/abandoned (with notes)

## WIP Limits

Recommended work-in-progress limits:
- **Per person**: 1-2 tasks maximum
- **Team total**: Depends on team size

Too many in-progress tasks indicates:
- ⚠️ Context switching
- ⚠️ Blocked work not being addressed
- ⚠️ Tasks larger than estimated

## How to Use This File

**For implementers**: Update when starting/finishing work
**For tracking**: Monitor task age and identify stuck work
**For standups**: Quick view of what's active

---

## Active Work

### V2-PLUGIN-001: Design Tool Context Helper Plugin

**Started**: 2025-10-14
**Branch**: `feat/v2-tool-context-plugin`
**Assignee**: Claude
**Priority**: Critical

**Status**: Design phase - investigating pain points and exploring solutions

**Quick Context**:
- Pain Score: 30/30 (validated across 100% of agents)
- Affects 9/9 context-aware tools
- Eliminates clone + move boilerplate in tool definitions

**See**: [backlog/by-status/ready.md](../by-status/ready.md#v2-plugin-001-design-tool-context-helper-plugin) for full task details

---

## Paused Work

*No paused tasks*

---

## Metadata

**Last updated**: 2025-10-14
**Last updated by**: V2-PLUGIN-001 started
**Total in-progress tasks**: 1
**Average time in progress**: N/A
**Longest in-progress task**: N/A

## Notes

If a task has been in progress for >5 days, consider:
- Is it blocked? Move to planned and document blocker
- Is it too large? Break into smaller tasks
- Does it need help? Pair programming or review
