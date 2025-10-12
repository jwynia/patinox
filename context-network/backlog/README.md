# Backlog Management

This directory contains the project's task backlog organized for optimal workflow.

## Directory Structure

```
backlog/
├── by-status/          # Primary view: Status-based workflow
│   ├── ready.md        # Ready for immediate work
│   ├── planned.md      # Groomed but blocked
│   ├── in-progress.md  # Currently being worked
│   └── completed.md    # Recently completed (14 days)
├── by-priority/        # Secondary view: Priority-based
│   ├── critical.md     # P0 - Critical
│   ├── high.md         # P1 - High
│   ├── medium.md       # P2 - Medium
│   └── low.md          # P3 - Low
└── archived/           # Historical completions
    └── YYYY-MM/        # Archived by month
```

## Workflow

### For Implementers

1. **Find next task**: `/next` → Suggests task from `by-status/ready.md`
2. **Start work**: `/implement [TASK-ID]` → Moves to `in-progress.md`
3. **Complete**: Merge PR → Moves to `completed.md` (via `/sync`)

### For Planners

1. **Plan feature**: `/plan [feature-name]` → Creates planning docs
2. **Groom backlog**: `/groom` → Populates `ready.md` or `planned.md`
3. **Track progress**: `/status` → Overview of all status files

### For Project Managers

1. **Check velocity**: Review `completed.md` for sprint accomplishments
2. **Identify blockers**: Check `planned.md` for dependency issues
3. **Monitor WIP**: Check `in-progress.md` for stuck work

## Task Lifecycle

```
Planning → Grooming → Ready → In Progress → Completed → Archived
              ↓          ↓          ↓            ↓           ↓
         planned.md  ready.md  in-progress  completed  archived/
```

## Views

### Status-Based (Primary)

Use `by-status/` for daily workflow:
- "What can I work on?" → `ready.md`
- "What's being worked on?" → `in-progress.md`
- "What did we accomplish?" → `completed.md`

### Priority-Based (Secondary)

Use `by-priority/` for planning:
- "What's most urgent?" → `critical.md`
- "What matters this sprint?" → `high.md`
- "What's nice to have?" → `medium.md`

Tasks appear in BOTH views (cross-referenced).

## Task IDs

All tasks have unique IDs:
- **FEAT-NNN**: Feature development
- **INFRA-NNN**: Infrastructure work
- **REFACTOR-NNN**: Code refactoring
- **BUG-NNN**: Bug fixes
- **DOCS-NNN**: Documentation
- **TEST-NNN**: Testing improvements
- **PERF-NNN**: Performance optimization

## Integration with Commands

- **`/sync`**: Detects completed work, updates status files
- **`/groom`**: Analyzes tasks, populates ready/planned files
- **`/next`**: Reads ready.md, suggests next task
- **`/implement`**: Moves task to in-progress
- **`/status`**: Aggregates across all status files
- **`/plan`**: Creates tasks with proper IDs

## See Also

- [Task templates](../tasks/templates/)
- [Migration decision](../decisions/backlog-structure-migration-2025-10-12.md)
- [Task management patterns](../tasks/task-management-patterns.md)
