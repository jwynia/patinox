# CLI Plugin - Design Document

**Status**: Design Phase
**Created**: 2025-11-13
**Task**: V2-PLUGIN-002-A
**Priority**: Critical (Pain Score: 30/30)

## Executive Summary

This plugin eliminates CLI argument parsing boilerplate that affects 100% of CLI-based agents. The design provides a type-safe, ergonomic API for argument parsing with automatic `--help` generation.

## Problem Statement

### Current Pain Point

Every CLI-based agent requires extensive manual argument parsing (30-35 lines):

```rust
// file_processor.rs - 30 lines of CLI boilerplate
let args: Vec<String> = std::env::args().collect();

if args.len() < 2 {
    print_usage(&args[0]);
    std::process::exit(1);
}

// Check for special flags
if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
    print_usage(&args[0]);
    return Ok(());
}

let file_path = &args[1];

let user_query = if args.len() > 2 {
    args[2..].join(" ")
} else {
    format!("Default query...")
};

// Plus 16+ lines for print_usage() function!
fn print_usage(program_name: &str) {
    println!("File Processor Agent - Analyze files with AI");
    println!();
    println!("USAGE:");
    println!("    {} <file-path> [query]", program_name);
    // ... 12 more lines
}
```

### Impact Analysis

**Validated Across Both Agents:**
- **V2-AGENT-001** (file_processor): 30 lines of CLI code
- **V2-AGENT-002** (doc_generator): 35 lines of CLI code
- **Pattern Frequency**: 100% (2/2 CLI-based agents)

**Pain Point Breakdown:**
1. Manual argument collection
2. Manual length/validation checking
3. Manual `--help` flag detection
4. Manual positional argument access
5. Manual optional argument handling with defaults
6. Manual flag parsing (`--output`, `-o`)
7. Manual help text generation (repetitive println!)

## Design Goals

### Must Have
1. ✅ Eliminate 80%+ of CLI boilerplate
2. ✅ Type-safe argument access
3. ✅ Automatic `--help` generation
4. ✅ Clear error messages for invalid args
5. ✅ Works before agent creation (args used in tool setup)

### Should Have
1. ✅ Support common types (String, PathBuf, bool)
2. ✅ Support positional and named arguments
3. ✅ Support required vs optional args
4. ✅ Support default values

### Nice to Have
1. ⚪ Subcommands (defer to future if needed)
2. ⚪ Argument validation (defer to future)
3. ⚪ Environment variable fallback

## Proposed Solution

### Option A: Separate CLI Args Parser (RECOMMENDED)

Parse CLI arguments separately, then use them to configure the agent:

```rust
use patinox::cli::CliArgs;

fn main() -> patinox::Result<()> {
    // Parse CLI args - handles --help automatically
    let cli = CliArgs::new("file_processor", "Analyze files with AI")
        .arg("file_path", "Path to file to process")
            .required()
        .arg("query", "What to analyze (optional)")
            .optional()
            .default("Analyze this file")
        .parse()?;

    // Access parsed arguments
    let file_path = cli.get("file_path");
    let query = cli.get("query");

    // Use in agent creation
    let agent = create_agent("file_processor")
        .tool_fn_with("read_file", "Read file", &file_path, |path, _| { ... });

    agent.run(query).await
}
```

**Pros:**
- Clean separation (parsing before agent creation)
- Args available for tool configuration
- Automatic `--help` handling
- Type-safe access with `get()`

**Cons:**
- Not integrated with Agent struct
- Requires separate import (`use patinox::cli::CliArgs`)

### Option B: Agent Plugin Approach

Integrate CLI parsing into the agent builder:

```rust
let agent = create_agent("file_processor")
    .with_cli(Cli::new()
        .arg("file_path").required()
        .arg("query").optional().default("Analyze")
    );

// Somewhere inside agent.run():
let file_path = agent.cli.get("file_path");
```

**Pros:**
- Integrated with Agent
- Uses familiar `.with_*()` pattern

**Cons:**
- Parsed args not available during agent construction
- Can't use args in `.tool_fn_with()` calls
- Awkward for our use case (args needed before agent is fully built)

### Recommendation: Option A

**Decision**: Use **Option A** (Separate CLI Args Parser)

**Rationale**:
1. In our examples, CLI args are used DURING agent creation (for tool context)
2. `--help` should be handled BEFORE creating the agent
3. Error messages for invalid args should show BEFORE agent initialization
4. Simpler mental model: Parse args → Use args → Create agent

## API Design

### Core Types

```rust
/// CLI argument parser and container
pub struct CliArgs {
    program_name: String,
    description: String,
    args: Vec<ArgSpec>,
    parsed: HashMap<String, String>,
}

/// Argument specification
struct ArgSpec {
    name: String,
    description: String,
    required: bool,
    default: Option<String>,
    arg_type: ArgType,
}

enum ArgType {
    Positional(usize), // Position index
    Flag(String),      // --flag or -f
}
```

### Builder API

```rust
impl CliArgs {
    /// Create new CLI args parser
    pub fn new(program_name: impl Into<String>, description: impl Into<String>) -> Self;

    /// Add a positional argument
    pub fn arg(mut self, name: impl Into<String>, description: impl Into<String>) -> ArgBuilder;

    /// Add a flag (--flag or -f)
    pub fn flag(mut self, name: impl Into<String>, short: char, description: impl Into<String>) -> Self;

    /// Parse arguments from std::env::args()
    pub fn parse(self) -> Result<Self, CliError>;

    /// Get parsed argument value
    pub fn get(&self, name: &str) -> &str;

    /// Check if flag is set
    pub fn is_set(&self, name: &str) -> bool;
}

/// Builder for argument specifications
pub struct ArgBuilder {
    args: CliArgs,
    current_arg: ArgSpec,
}

impl ArgBuilder {
    /// Mark argument as required
    pub fn required(mut self) -> CliArgs;

    /// Mark argument as optional
    pub fn optional(mut self) -> CliArgs;

    /// Set default value for optional argument
    pub fn default(mut self, value: impl Into<String>) -> CliArgs;
}
```

### Usage Examples

#### Example 1: file_processor (Before)

**Current (30 lines)**:
```rust
let args: Vec<String> = std::env::args().collect();

if args.len() < 2 {
    print_usage(&args[0]);
    std::process::exit(1);
}

if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
    print_usage(&args[0]);
    return Ok(());
}

let file_path = &args[1];

let user_query = if args.len() > 2 {
    args[2..].join(" ")
} else {
    format!("Please analyze the file '{}'...", file_path)
};

fn print_usage(program_name: &str) {
    println!("File Processor Agent - Analyze files with AI");
    println!();
    println!("USAGE:");
    println!("    {} <file-path> [query]", program_name);
    println!();
    // ... 12 more lines
}
```

**With CLI Plugin (8 lines)**:
```rust
use patinox::cli::CliArgs;

let cli = CliArgs::new("file_processor", "Analyze files with AI")
    .arg("file_path", "Path to file to process")
        .required()
    .arg("query", "What to analyze (optional)")
        .optional()
        .default("Please analyze this file...")
    .parse()?;

let file_path = cli.get("file_path");
let query = cli.get("query");
```

**Reduction**: 30 lines → 8 lines (73% reduction!)

#### Example 2: doc_generator (Before)

**Current (35 lines)**:
```rust
let args: Vec<String> = std::env::args().collect();

if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
    print_usage(&args[0]);
    return Ok(());
}

let source_path = &args[1];

let output_path = if let Some(idx) = args.iter().position(|a| a == "--output" || a == "-o") {
    args.get(idx + 1).map(|s| s.to_string())
} else {
    None
};

fn print_usage(program_name: &str) {
    println!("Documentation Generator - Generate docs from Rust code");
    println!();
    println!("USAGE:");
    println!("    {} <source-file> [OPTIONS]", program_name);
    println!();
    // ... 14 more lines
}
```

**With CLI Plugin (10 lines)**:
```rust
use patinox::cli::CliArgs;

let cli = CliArgs::new("doc_generator", "Generate docs from Rust code")
    .arg("source_path", "Rust source file to document")
        .required()
    .flag("output", 'o', "Output file path (optional)")
    .parse()?;

let source_path = cli.get("source_path");
let output_path = cli.get_optional("output");
```

**Reduction**: 35 lines → 10 lines (71% reduction!)

### Automatic Help Generation

The `CliArgs::parse()` method automatically handles `--help` and `-h` flags:

```bash
$ file_processor --help
file_processor - Analyze files with AI

USAGE:
    file_processor <file_path> [query]

ARGUMENTS:
    <file_path>    Path to file to process (required)
    [query]        What to analyze (optional)
                   Default: Please analyze this file...

OPTIONS:
    -h, --help     Show this help message

EXAMPLES:
    file_processor README.md
    file_processor src/main.rs "What does this do?"
```

The help text is generated automatically from:
- Program name (from `CliArgs::new()`)
- Description (from `CliArgs::new()`)
- Arg names and descriptions (from `.arg()`)
- Required/optional status (from `.required()`/`.optional()`)
- Default values (from `.default()`)

### Error Handling

```rust
pub enum CliError {
    MissingRequired(String),  // "Missing required argument: file_path"
    InvalidFlag(String),      // "Unknown flag: --foo"
    ParseError(String),       // Generic parse error
}

impl CliArgs {
    pub fn parse(self) -> Result<Self, CliError> {
        // 1. Check for --help/-h first
        // 2. Validate all required args present
        // 3. Parse flags and positional args
        // 4. Apply defaults for missing optional args
        // 5. Return Err(CliError::*) if validation fails
    }
}
```

Error messages are clear and actionable:
```
Error: Missing required argument: file_path

USAGE:
    file_processor <file_path> [query]

Run 'file_processor --help' for more information.
```

## Implementation Plan

### Phase 1: Core Types & Builder (2-3 hours)
- Define `CliArgs`, `ArgSpec`, `ArgBuilder` structs
- Implement builder pattern (`.arg()`, `.flag()`, `.required()`, etc.)
- Implement basic `get()` accessor

### Phase 2: Parsing Logic (2-3 hours)
- Implement `parse()` method
- Handle `--help` / `-h` automatically
- Parse positional arguments
- Parse flags
- Validate required arguments
- Apply defaults

### Phase 3: Help Generation (1-2 hours)
- Auto-generate help text from arg specs
- Format output cleanly
- Include examples section (optional)

### Phase 4: Testing (2-3 hours)
- Unit tests for argument parsing
- Unit tests for validation
- Unit tests for help generation
- Integration tests with example agents

### Phase 5: Migration (1-2 hours)
- Refactor file_processor.rs to use CLI plugin
- Refactor doc_generator.rs to use CLI plugin
- Verify both examples work correctly

## Files to Create/Modify

**New Files**:
- `src/cli.rs` - CLI args parser implementation
- `src/cli/args.rs` - CliArgs struct and methods
- `src/cli/error.rs` - CliError types
- `examples/cli_demo.rs` - Example showing CLI plugin usage

**Modified Files**:
- `src/lib.rs` - Export `cli` module
- `examples/file_processor.rs` - Refactor to use CliArgs
- `examples/doc_generator.rs` - Refactor to use CliArgs

## Success Metrics

- [ ] Reduces CLI boilerplate from ~30 lines to ~10 lines (70% reduction)
- [ ] Automatic `--help` generation works
- [ ] Clear error messages for missing arguments
- [ ] Both example agents refactored successfully
- [ ] cargo test passes (all existing + new CLI tests)
- [ ] cargo clippy passes (zero warnings)

## Validation Against Examples

### file_processor Current CLI Code
- ✅ Positional arg: `file_path`
- ✅ Optional arg with default: `query`
- ✅ Help flag handling: `--help`, `-h`
- ✅ Help text generation

### doc_generator Current CLI Code
- ✅ Positional arg: `source_path`
- ✅ Optional flag: `--output` / `-o`
- ✅ Help flag handling
- ✅ Help text generation

**Coverage**: Design covers 100% of patterns in both examples ✅

## Design Decisions Record

### Decision 1: Separate Parser vs Integrated Plugin

**Options Considered**:
- A) Separate `CliArgs::parse()` before agent creation
- B) Integrated `.with_cli()` plugin

**Decision**: Option A (Separate Parser)

**Rationale**:
- Args needed DURING agent construction (for tool context)
- `--help` should handle before agent initialization
- Clearer separation of concerns

### Decision 2: Type System for Arguments

**Options Considered**:
- A) String-only, users convert as needed
- B) Generic `get<T>()` with FromStr
- C) Specific methods: `get_string()`, `get_path()`, `get_int()`

**Decision**: Start with A (String-only), add B later if needed

**Rationale**:
- All our examples use String or convert from String
- Simpler implementation for MVP
- Can add `get<T>()` in PLUGIN-002-B if validated as needed

### Decision 3: Help Text Location

**Options Considered**:
- A) Auto-generate from arg specs
- B) Require users to provide custom help text
- C) Hybrid (auto-generate with override option)

**Decision**: Option A (Auto-generate)

**Rationale**:
- Reduces boilerplate (no manual println! chains)
- Help text always in sync with arg specs
- Can add override later if needed

## Migration Path

### Step-by-Step Migration (file_processor example)

**Step 1**: Add CLI args parsing
```rust
use patinox::cli::CliArgs;

let cli = CliArgs::new("file_processor", "Analyze files with AI")
    .arg("file_path", "Path to file to process").required()
    .arg("query", "What to analyze").optional().default("Analyze this file")
    .parse()?;
```

**Step 2**: Replace manual arg access
```rust
// Before
let file_path = &args[1];
let query = if args.len() > 2 { args[2..].join(" ") } else { ... };

// After
let file_path = cli.get("file_path");
let query = cli.get("query");
```

**Step 3**: Remove old boilerplate
```rust
// Delete these:
let args: Vec<String> = std::env::args().collect();
if args.len() < 2 { print_usage(&args[0]); std::process::exit(1); }
if args.contains(&"--help".to_string()) { ... }
fn print_usage(...) { ... }
```

**Step 4**: Test and validate
```bash
cargo run --example file_processor -- --help
cargo run --example file_processor -- README.md
cargo run --example file_processor -- src/main.rs "Analyze this"
```

## Future Enhancements (Post-MVP)

Defer to PLUGIN-002-B or future tasks if validated as needed:
- Typed argument access: `cli.get::<PathBuf>("file_path")`
- Argument validation: `.validate(|v| v.ends_with(".rs"))`
- Environment variable fallback: `.env("FILE_PATH")`
- Subcommands: `cli.subcommand("analyze").arg(...)`
- Short flags: `-f` instead of `--file`
- Multiple values: `--input file1.txt --input file2.txt`

## See Also

- [records/pain-points-file-processor-2025-10-13.md](../records/pain-points-file-processor-2025-10-13.md) - Pain Point #2
- [records/pain-points-doc-generator-2025-10-13.md](../records/pain-points-doc-generator-2025-10-13.md) - Pain Point #2
- [upcoming-post-layer-2.5.md](../backlog/by-status/upcoming-post-layer-2.5.md) - PLUGIN-002 series overview
