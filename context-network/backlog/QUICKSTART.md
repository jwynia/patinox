# Backlog Workflow Quick Start

**TL;DR**: Status-based workflow for managing tasks from planning to completion.

## For Implementers: "What Should I Work On?"

```bash
# Get next task suggestion
/next

# Output will show:
# **Next Task:** DOCS-001 - Document Backlog Structure Migration
# **Priority:** High
# **Size:** Small
# **Branch:** docs/backlog-structure-migration
# Start with: /implement DOCS-001
```

## For Planners: "How Do I Add New Work?"

### Planning a New Feature

```bash
# Step 1: Create planning documents
/plan implement-v2-minimal-agent

# This creates:
# - context-network/planning/implement-v2-minimal-agent/
#   - problem-definition.md
#   - task-breakdown.md (with task IDs!)
#   - risk-assessment.md
```

### Grooming Tasks

```bash
# Step 2: Groom backlog to make tasks actionable
/groom

# This:
# - Scans all task sources
# - Validates readiness
# - Populates backlog/by-status/ready.md or planned.md
# - Identifies blockers
```

### Checking Status

```bash
# See overall project status
/status

# See what's being worked on
cat context-network/backlog/by-status/in-progress.md

# See what's been completed
cat context-network/backlog/by-status/completed.md
```

## Task Lifecycle

```
Plan → Groom → Ready → In Progress → Completed → Archived
  ↓       ↓        ↓         ↓             ↓          ↓
/plan  /groom  ready.md  in-progress  completed  archived/
```

## File Locations

```
context-network/
├── backlog/
│   ├── by-status/
│   │   ├── ready.md          ← Start here for next task
│   │   ├── planned.md        ← Groomed but blocked
│   │   ├── in-progress.md    ← Currently active
│   │   └── completed.md      ← Recently done
│   └── archived/
│       └── 2025-10/          ← Historical record
├── tasks/
│   ├── FEAT-001.md           ← Individual task details
│   ├── INFRA-001.md
│   └── templates/
│       └── task-template.md  ← Copy this for new tasks
└── planning/
    └── [feature-name]/       ← Feature-level plans
```

## Task ID Format

- **FEAT-NNN**: New features
- **INFRA-NNN**: Infrastructure work
- **REFACTOR-NNN**: Code refactoring
- **BUG-NNN**: Bug fixes
- **DOCS-NNN**: Documentation
- **TEST-NNN**: Testing improvements
- **PERF-NNN**: Performance optimization

## Common Workflows

### Daily Work Session

```bash
# 1. Sync with reality
/sync

# 2. Check status
/status --brief

# 3. Get next task
/next

# 4. Start implementation
/implement FEAT-042
```

### Sprint Planning

```bash
# 1. Sync recent work
/sync --last 14d

# 2. Groom backlog
/groom --generate-sprint

# 3. Review ready tasks
cat context-network/backlog/by-status/ready.md
```

### Feature Planning

```bash
# 1. Plan the feature
/plan new-authentication-system

# 2. Review planning docs
# (Check context-network/planning/new-authentication-system/)

# 3. Groom to create actionable tasks
/groom --domain auth

# 4. Check what's ready
cat context-network/backlog/by-status/ready.md
```

## Creating a Task Manually

If you need to create a task outside of `/plan`:

```bash
# 1. Copy template
cp context-network/tasks/templates/task-template.md \
   context-network/tasks/FEAT-042.md

# 2. Fill in details
# (Edit the file with task information)

# 3. Add to ready.md
# (Add summary to backlog/by-status/ready.md)

# 4. Verify with /next
/next
```

## Slash Commands Reference

| Command | Purpose | Output |
|---------|---------|--------|
| `/next` | Suggest next task | Task ID and how to start |
| `/groom` | Make backlog actionable | Updates status files |
| `/sync` | Detect completed work | Updates task status |
| `/plan` | Design new feature | Planning docs + tasks |
| `/implement` | Start working on task | Moves to in-progress |
| `/status` | Project health check | Status report |

## What's Different from Before?

### Old Way (Category-Based)
```
tasks/
├── features/
├── refactoring/
└── performance/
```
- Hard to find what's ready to work on
- No clear task lifecycle
- Status scattered across files

### New Way (Status-Based)
```
backlog/by-status/
├── ready.md        ← Clear list of available work
├── planned.md      ← Clear list of blocked work
├── in-progress.md  ← Clear list of active work
└── completed.md    ← Clear list of done work
```
- Instant visibility into what's available
- Clear task lifecycle tracking
- Status is centralized and clear

## Need Help?

- **Full migration details**: See [decisions/backlog-structure-migration-2025-10-12.md](../decisions/backlog-structure-migration-2025-10-12.md)
- **Migration status**: See [MIGRATION-STATUS.md](./MIGRATION-STATUS.md)
- **Detailed README**: See [README.md](./README.md)
- **Task template**: See [../tasks/templates/task-template.md](../tasks/templates/task-template.md)

## Tips

1. **Use `/next`** instead of manually browsing - it applies smart selection logic
2. **Update status files** as tasks progress - keeps everyone in sync
3. **Run `/sync`** regularly - catches completed work automatically
4. **Run `/groom`** before sprints - ensures tasks are actionable
5. **Read task details** in `tasks/[TASK-ID].md` - status files have summaries only
