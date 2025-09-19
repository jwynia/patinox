# Memory Management Components Index

**Classification**: Location Index  
**Domain**: System Architecture  
**Last Updated**: 2025-01-20

## Purpose
Comprehensive index of memory management functionality across the Patinox codebase, including RAII patterns, resource cleanup, and lifecycle management.

## Core Components

### AsyncResourceGuard (`src/memory/resource.rs`)
**Purpose**: RAII guard for async resource cleanup  
**Key Types**: `AsyncResourceGuard<T>`, `CleanupFn<T>`, `CleanupError`, `CleanupPriority`  
**Dependencies**: tokio runtime for async cleanup via Drop trait  
**Testing**: `tests/resource_management_test.rs` (18 test cases)

**Critical Patterns**:
- Drop trait with tokio::spawn for non-blocking cleanup
- Send + 'static bounds for thread safety
- Type-safe resource wrapping with ownership transfer

### ResourceRegistry (`src/memory/registry.rs`)  
**Purpose**: Centralized resource tracking and cleanup coordination  
**Key Types**: `ResourceRegistry`, `ResourceInfo`, `CleanupRequest`  
**Dependencies**: Monitor trait for observability integration  
**Testing**: `tests/registry_integration_test.rs` (7 test cases)

**Critical Patterns**:
- Priority-based cleanup queue using BinaryHeap
- Background cleanup task with graceful shutdown
- Arc<RwLock<HashMap>> for thread-safe registry state

### Resource Identification (`src/memory/resource.rs:17-32`)
**Purpose**: Unique resource identification across system  
**Key Types**: `ResourceId` (UUID-based)  
**Usage**: Resource tracking, cleanup coordination, monitoring correlation  
**Properties**: Copy, Clone, Hash, Serialize, Deserialize

## Error Handling Architecture

### CleanupError Types (`src/memory/resource.rs:49-83`)
- `Timeout`: Cleanup operation exceeded time limit
- `AlreadyCleanedUp`: Resource was already processed  
- `Failed(Box<Error>)`: Cleanup operation encountered error
- `ShuttingDown`: Registry is shutting down

**Recovery Strategies**:
- `Timeout` → `Retry`
- `AlreadyCleanedUp` → `Fail` 
- `Failed(_)` → `Fallback`
- `ShuttingDown` → `Fail`

### Integration with PatinoxError (`src/memory/resource.rs:80-88`)
All CleanupError variants convert to `PatinoxError::Execution(ExecutionError::ResourceExhausted)`

## Testing Architecture

### Test Utilities Location
**Primary**: Individual test files (currently duplicated)  
**Utilities**: TestResource, TestMonitor/MockMonitor implementations  
**Constants**: Test timeouts and delay values  
**Future**: Consolidation into `tests/common/mod.rs` (see task)

### Test Categories
1. **Unit Tests** (`src/memory/*/tests` modules): Component-specific functionality
2. **Integration Tests** (`tests/memory_integration_test.rs`): Component interaction
3. **Resource Management Tests** (`tests/resource_management_test.rs`): AsyncResourceGuard scenarios  
4. **Registry Integration Tests** (`tests/registry_integration_test.rs`): ResourceRegistry functionality

### Property-Based Testing (`tests/resource_management_test.rs:427-452`)
**Location**: `property_based_tests` module  
**Coverage**: Resource creation invariants, basic property verification  
**Tools**: proptest crate for property generation
**Status**: Needs improvement (see task: weak properties identified)

## Configuration and Constants

### Timeout Values (`src/memory/registry.rs:19-21`)
- `SHUTDOWN_GRACE_PERIOD_MS`: 100ms - Time to wait for cleanup during shutdown
- `CLEANUP_POLL_INTERVAL_MS`: 10ms - Background task polling interval

### Test Constants (`tests/resource_management_test.rs:14-17`)  
- `ASYNC_CLEANUP_WAIT_MS`: 50ms - Time to wait for async cleanup in tests
- `SHORT_TIMEOUT_MS`: 10ms - Short timeout for timeout testing  
- `VERY_SHORT_DELAY_MS`: 1ms - Minimal delay for simulating work

## Future Integration Points

### Monitoring Integration (TODO)
**Blocker**: Need MonitorEventType extension for resource events  
**Location**: Scattered TODO comments in registry.rs  
**Integration Points**: Resource registration, cleanup completion, failure events

### Caching Layer (Planned)
**Location**: TBD (`src/memory/cache.rs` likely)  
**Dependencies**: AsyncResourceGuard for cache entry cleanup  
**Integration**: ResourceRegistry for cache resource tracking

### Connection Pooling (Planned)  
**Location**: TBD (`src/memory/pool.rs` likely)
**Dependencies**: AsyncResourceGuard for connection cleanup, ResourceRegistry for pool monitoring  
**Pattern**: Similar to cleanup coordination but with connection reuse

## Related Documentation

**Architecture Decisions**:
- [2025-01-20-001-async-drop-pattern](../discovery/2025-01-20-001-async-drop-pattern.md) - Drop trait async cleanup approach
- [2025-01-20-002-priority-queue-cleanup](../discovery/2025-01-20-002-priority-queue-cleanup.md) - BinaryHeap priority queue implementation

**Implementation Notes**:
- [[Memory Management Design Decisions]] - Trade-offs and alternatives considered
- [[TDD Memory Management Workflow]] - Development process and testing approach

**Future Work**:
- [[Memory Management Phase 2 Planning]] - Advanced features roadmap  
- [[Monitoring Integration Design]] - Resource event monitoring approach

## Navigation Paths

**From Architecture Overview** → Memory Management → This Index  
**From Error Handling** → CleanupError → Resource Management  
**From Testing Strategy** → Async Testing → Memory Management Tests  
**From Performance Optimization** → Resource Cleanup → Memory Management

**To Implementation Details** → Individual component files  
**To Testing** → Test file index and utilities  
**To Future Features** → Phase 2 planning documents