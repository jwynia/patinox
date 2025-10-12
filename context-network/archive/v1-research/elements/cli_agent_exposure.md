# CLI Agent Exposure Architecture

## Overview

This document defines the architecture for exposing Patinox agents through command-line interfaces, emphasizing local-first operation with optional protocol connectivity. The design follows the "freedom without obligation" philosophy - agents can connect to other systems but don't require external dependencies to function.

## Core Philosophy: Local-First, Connected-Optional

CLI agents should be:

1. **Self-contained**: Fully functional without network access
2. **Resource-aware**: Direct access to local filesystem, containers, and system resources
3. **Optionally connected**: Can reach out to other agents/tools when available
4. **Container-native**: Designed for devcontainer and Docker environments
5. **Pipeline-friendly**: Composable with Unix pipes and scripts

## CLI Architecture Layers

### 1. Command Structure

```rust
/// Core CLI application structure
pub struct PatinoxCLI {
    /// Local agent registry
    local_agents: LocalAgentRegistry,
    
    /// Optional remote connections
    remote_connections: Option<RemoteConnections>,
    
    /// Resource access layer
    resources: ResourceAccess,
    
    /// Configuration
    config: CLIConfig,
}

/// Main CLI entry point using clap
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Global verbosity level
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    /// Run in offline mode (no network connections)
    #[arg(long, global = true)]
    offline: bool,
    
    /// Output format
    #[arg(short, long, global = true, value_enum, default_value = "auto")]
    format: OutputFormat,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Execute an agent
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },
    
    /// Run a tool
    Tool {
        #[command(subcommand)]
        action: ToolCommands,
    },
    
    /// Execute a workflow
    Workflow {
        #[command(subcommand)]
        action: WorkflowCommands,
    },
    
    /// Manage resources
    Resource {
        #[command(subcommand)]
        action: ResourceCommands,
    },
    
    /// Interactive REPL mode
    Repl {
        /// Agent to use in REPL
        #[arg(short, long)]
        agent: Option<String>,
    },
    
    /// Serve agents via protocols
    Serve {
        /// Protocols to enable
        #[arg(short, long, value_delimiter = ',')]
        protocols: Vec<Protocol>,
        
        /// Port to listen on
        #[arg(short = 'P', long, default_value = "8080")]
        port: u16,
    },
}

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    Auto,
    Json,
    Yaml,
    Text,
    Table,
    Markdown,
}
```

### 2. Local Resource Access

Direct access to filesystem and container resources:

```rust
/// Resource access layer for local operations
pub struct ResourceAccess {
    /// Filesystem access with sandboxing
    filesystem: FilesystemAccess,
    
    /// Container runtime access
    container: Option<ContainerAccess>,
    
    /// Process management
    process: ProcessManager,
    
    /// System information
    system: SystemInfo,
}

pub struct FilesystemAccess {
    /// Allowed paths for read access
    read_paths: Vec<PathBuf>,
    
    /// Allowed paths for write access
    write_paths: Vec<PathBuf>,
    
    /// Current working directory
    cwd: PathBuf,
}

impl FilesystemAccess {
    pub async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        // Check if path is allowed
        if !self.is_readable(path) {
            return Err(Error::PermissionDenied(path.to_path_buf()));
        }
        
        tokio::fs::read(path).await
            .map_err(|e| Error::FileRead(path.to_path_buf(), e))
    }
    
    pub async fn write_file(&self, path: &Path, content: &[u8]) -> Result<()> {
        // Check if path is allowed
        if !self.is_writable(path) {
            return Err(Error::PermissionDenied(path.to_path_buf()));
        }
        
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(path, content).await
            .map_err(|e| Error::FileWrite(path.to_path_buf(), e))
    }
    
    pub fn watch(&self, path: &Path) -> impl Stream<Item = FsEvent> {
        // Set up file watcher for reactive operations
        let (tx, rx) = mpsc::channel(100);
        let watcher = notify::recommended_watcher(move |event| {
            let _ = tx.blocking_send(event);
        }).unwrap();
        
        ReceiverStream::new(rx)
    }
}

/// Container runtime access for devcontainer scenarios
pub struct ContainerAccess {
    runtime: ContainerRuntime,
    current_container: Option<ContainerId>,
}

impl ContainerAccess {
    pub async fn exec_in_container(
        &self,
        container: &ContainerId,
        command: &[String],
    ) -> Result<ExecResult> {
        match &self.runtime {
            ContainerRuntime::Docker => {
                self.docker_exec(container, command).await
            }
            ContainerRuntime::Podman => {
                self.podman_exec(container, command).await
            }
            ContainerRuntime::DevContainer => {
                self.devcontainer_exec(container, command).await
            }
        }
    }
    
    pub async fn mount_volume(
        &self,
        host_path: &Path,
        container_path: &Path,
    ) -> Result<()> {
        // Mount host filesystem into container
        self.runtime.mount(host_path, container_path).await
    }
}
```

### 3. Agent Execution Modes

Multiple execution modes for different use cases:

```rust
/// Different modes of agent execution
pub enum ExecutionMode {
    /// Single command execution
    OneShot {
        agent: AgentId,
        task: String,
        params: HashMap<String, Value>,
    },
    
    /// Interactive REPL session
    Interactive {
        agent: AgentId,
        history_file: Option<PathBuf>,
    },
    
    /// Pipeline mode (stdin/stdout)
    Pipeline {
        agent: AgentId,
        input_format: DataFormat,
        output_format: DataFormat,
    },
    
    /// Batch processing
    Batch {
        agent: AgentId,
        input_file: PathBuf,
        output_file: Option<PathBuf>,
    },
    
    /// Watch mode (reactive)
    Watch {
        agent: AgentId,
        paths: Vec<PathBuf>,
        patterns: Vec<String>,
    },
    
    /// Daemon mode
    Daemon {
        agent: AgentId,
        pid_file: PathBuf,
        log_file: Option<PathBuf>,
    },
}

impl ExecutionMode {
    pub async fn execute(&self, cli: &PatinoxCLI) -> Result<()> {
        match self {
            Self::OneShot { agent, task, params } => {
                let agent = cli.get_agent(agent)?;
                let result = agent.execute_task(task, params).await?;
                cli.output_result(result)
            }
            
            Self::Interactive { agent, history_file } => {
                let agent = cli.get_agent(agent)?;
                cli.run_interactive_session(agent, history_file).await
            }
            
            Self::Pipeline { agent, input_format, output_format } => {
                let agent = cli.get_agent(agent)?;
                cli.run_pipeline_mode(agent, input_format, output_format).await
            }
            
            Self::Watch { agent, paths, patterns } => {
                let agent = cli.get_agent(agent)?;
                cli.run_watch_mode(agent, paths, patterns).await
            }
            
            // ... other modes
        }
    }
}
```

### 4. Interactive REPL

Rich interactive experience for agent interaction:

```rust
/// Interactive REPL for agent interaction
pub struct AgentREPL {
    agent: Box<dyn Agent>,
    prompt: String,
    history: History,
    completer: AgentCompleter,
    context: ReplContext,
}

impl AgentREPL {
    pub async fn run(&mut self) -> Result<()> {
        let mut rl = Editor::<AgentCompleter>::new()?;
        rl.set_helper(Some(self.completer.clone()));
        
        if let Some(history_path) = &self.context.history_file {
            rl.load_history(history_path)?;
        }
        
        loop {
            let readline = rl.readline(&self.prompt);
            
            match readline {
                Ok(line) => {
                    rl.add_history_entry(&line);
                    
                    match self.process_command(&line).await {
                        Ok(CommandResult::Exit) => break,
                        Ok(CommandResult::Output(output)) => {
                            self.display_output(output);
                        }
                        Ok(CommandResult::Continue) => continue,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        if let Some(history_path) = &self.context.history_file {
            rl.save_history(history_path)?;
        }
        
        Ok(())
    }
    
    async fn process_command(&mut self, input: &str) -> Result<CommandResult> {
        // Handle special commands
        if input.starts_with('/') {
            return self.handle_repl_command(input).await;
        }
        
        // Execute as agent task
        let result = self.agent.execute_string(input).await?;
        Ok(CommandResult::Output(result))
    }
    
    async fn handle_repl_command(&mut self, command: &str) -> Result<CommandResult> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0).map(|s| *s) {
            Some("/help") => {
                self.show_help();
                Ok(CommandResult::Continue)
            }
            Some("/context") => {
                self.show_context();
                Ok(CommandResult::Continue)
            }
            Some("/clear") => {
                self.context.clear();
                Ok(CommandResult::Continue)
            }
            Some("/save") => {
                let path = parts.get(1).ok_or(Error::MissingArgument)?;
                self.save_session(Path::new(path)).await?;
                Ok(CommandResult::Continue)
            }
            Some("/load") => {
                let path = parts.get(1).ok_or(Error::MissingArgument)?;
                self.load_session(Path::new(path)).await?;
                Ok(CommandResult::Continue)
            }
            Some("/exit") | Some("/quit") => {
                Ok(CommandResult::Exit)
            }
            _ => Err(Error::UnknownCommand(command.to_string()))
        }
    }
}
```

### 5. Pipeline Integration

Unix-style pipeline support:

```rust
/// Pipeline mode for Unix-style composition
pub struct PipelineMode {
    agent: Box<dyn Agent>,
    input_format: DataFormat,
    output_format: DataFormat,
    buffer_size: usize,
}

impl PipelineMode {
    pub async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        
        let mut reader = BufReader::new(stdin);
        let mut writer = BufWriter::new(stdout);
        
        // Process input line by line or in chunks
        match self.input_format {
            DataFormat::JsonLines => {
                let mut line = String::new();
                while reader.read_line(&mut line).await? > 0 {
                    let input: Value = serde_json::from_str(&line)?;
                    let output = self.agent.process(input).await?;
                    
                    self.write_output(&mut writer, output).await?;
                    line.clear();
                }
            }
            DataFormat::Json => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).await?;
                let input: Value = serde_json::from_slice(&buffer)?;
                let output = self.agent.process(input).await?;
                self.write_output(&mut writer, output).await?;
            }
            DataFormat::Csv => {
                let mut csv_reader = csv_async::AsyncReader::from_reader(reader);
                let mut csv_writer = csv_async::AsyncWriter::from_writer(writer);
                
                // Process CSV records
                let mut records = csv_reader.records();
                while let Some(record) = records.next().await {
                    let record = record?;
                    let input = self.csv_to_value(record)?;
                    let output = self.agent.process(input).await?;
                    self.write_csv_output(&mut csv_writer, output).await?;
                }
            }
            // ... other formats
        }
        
        Ok(())
    }
}

#[derive(Clone)]
pub enum DataFormat {
    Json,
    JsonLines,
    Csv,
    Yaml,
    Text,
    Binary,
}
```

### 6. Offline/Online Mode Switching

Seamless transition between connected and disconnected operation:

```rust
/// Connection manager for offline/online modes
pub struct ConnectionManager {
    mode: Arc<RwLock<ConnectionMode>>,
    remote_agents: Arc<RwLock<HashMap<AgentId, RemoteAgent>>>,
    discovery: ServiceDiscovery,
    retry_policy: RetryPolicy,
}

#[derive(Clone)]
pub enum ConnectionMode {
    /// Fully offline - no network attempts
    Offline,
    
    /// Online - actively connected
    Online {
        connections: Vec<Connection>,
    },
    
    /// Hybrid - try remote, fall back to local
    Hybrid {
        preference: ConnectionPreference,
    },
    
    /// Opportunistic - use remote when available
    Opportunistic,
}

impl ConnectionManager {
    pub async fn execute_with_fallback(
        &self,
        agent_id: &AgentId,
        task: Task,
    ) -> Result<Value> {
        let mode = self.mode.read().await;
        
        match &*mode {
            ConnectionMode::Offline => {
                // Use local agent only
                self.execute_local(agent_id, task).await
            }
            
            ConnectionMode::Online { connections } => {
                // Try remote first, fail if not available
                match self.execute_remote(agent_id, task).await {
                    Ok(result) => Ok(result),
                    Err(e) if e.is_network() => {
                        Err(Error::RemoteUnavailable(agent_id.clone()))
                    }
                    Err(e) => Err(e),
                }
            }
            
            ConnectionMode::Hybrid { preference } => {
                match preference {
                    ConnectionPreference::PreferRemote => {
                        // Try remote first, fall back to local
                        self.execute_remote(agent_id, task.clone()).await
                            .or_else(|_| self.execute_local(agent_id, task).await)
                    }
                    ConnectionPreference::PreferLocal => {
                        // Try local first, fall back to remote
                        self.execute_local(agent_id, task.clone()).await
                            .or_else(|_| self.execute_remote(agent_id, task).await)
                    }
                }
            }
            
            ConnectionMode::Opportunistic => {
                // Check if remote is available
                if self.is_remote_available(agent_id).await {
                    self.execute_remote(agent_id, task).await
                } else {
                    self.execute_local(agent_id, task).await
                }
            }
        }
    }
    
    /// Monitor network status and update mode
    pub async fn monitor_connectivity(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let reachable = self.check_remote_endpoints().await;
            
            let mut mode = self.mode.write().await;
            match &*mode {
                ConnectionMode::Opportunistic => {
                    // Update available connections
                    if let Some(connections) = reachable {
                        *mode = ConnectionMode::Online { connections };
                    }
                }
                ConnectionMode::Online { .. } if reachable.is_none() => {
                    // Lost connectivity
                    warn!("Lost remote connectivity, switching to offline mode");
                    *mode = ConnectionMode::Offline;
                }
                _ => {}
            }
        }
    }
}
```

### 7. Shell Integration

Integration with shell environments:

```rust
/// Shell completion generator
pub struct ShellCompletion;

impl ShellCompletion {
    pub fn generate(shell: Shell) -> String {
        match shell {
            Shell::Bash => Self::generate_bash(),
            Shell::Zsh => Self::generate_zsh(),
            Shell::Fish => Self::generate_fish(),
            Shell::PowerShell => Self::generate_powershell(),
        }
    }
    
    fn generate_bash() -> String {
        r#"
_patinox_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Get available commands from patinox
    if [ $COMP_CWORD -eq 1 ]; then
        opts=$(patinox completions list-commands)
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    # Get context-aware completions
    opts=$(patinox completions suggest "${COMP_WORDS[@]}")
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
}

complete -F _patinox_completions patinox
        "#.to_string()
    }
}

/// Shell alias generator for common patterns
pub struct ShellAlias;

impl ShellAlias {
    pub fn generate_aliases() -> Vec<(String, String)> {
        vec![
            ("px".into(), "patinox".into()),
            ("pxa".into(), "patinox agent".into()),
            ("pxt".into(), "patinox tool".into()),
            ("pxw".into(), "patinox workflow".into()),
            ("pxr".into(), "patinox repl".into()),
        ]
    }
}
```

### 8. Configuration Management

Devcontainer-aware configuration:

```rust
/// CLI configuration with devcontainer support
pub struct CLIConfig {
    /// Global settings
    global: GlobalConfig,
    
    /// Per-agent settings
    agents: HashMap<AgentId, AgentConfig>,
    
    /// Connection settings
    connections: ConnectionConfig,
    
    /// Resource limits
    resources: ResourceConfig,
}

impl CLIConfig {
    pub async fn load() -> Result<Self> {
        // Priority order for configuration
        let sources = vec![
            // 1. Environment variables (highest priority)
            ConfigSource::Environment,
            
            // 2. Local .patinox folder in current directory
            ConfigSource::LocalFolder(PathBuf::from(".patinox")),
            
            // 3. Workspace .patinox folder (for devcontainers)
            ConfigSource::WorkspaceFolder(Self::find_workspace_root()?),
            
            // 4. User home directory (avoid in containers)
            ConfigSource::UserHome,
            
            // 5. System-wide configuration (lowest priority)
            ConfigSource::System,
        ];
        
        let mut config = Self::default();
        
        for source in sources {
            if let Some(partial) = source.load().await? {
                config.merge(partial)?;
            }
        }
        
        Ok(config)
    }
    
    fn find_workspace_root() -> Result<PathBuf> {
        // Check for devcontainer markers
        if let Ok(workspace) = env::var("WORKSPACE_FOLDER") {
            return Ok(PathBuf::from(workspace));
        }
        
        // Check for .devcontainer folder
        let mut current = env::current_dir()?;
        loop {
            if current.join(".devcontainer").exists() {
                return Ok(current);
            }
            
            if !current.pop() {
                break;
            }
        }
        
        // Fall back to current directory
        env::current_dir().map_err(Into::into)
    }
}
```

## Example CLI Usage Patterns

### Basic Agent Execution
```bash
# One-shot execution
patinox agent execute reasoning "Explain quantum computing"

# With parameters
patinox agent execute data-processor \
  --param input_file=data.csv \
  --param output_format=json

# Pipeline mode
cat data.json | patinox agent execute transformer --format json | jq .
```

### Interactive Sessions
```bash
# Start REPL with specific agent
patinox repl --agent researcher

# Interactive session with history
patinox repl --agent assistant --history ~/.patinox_history
```

### Workflow Execution
```bash
# Run workflow from file
patinox workflow run analysis_pipeline.yaml --input data/

# Watch mode
patinox workflow watch --paths src/ --pattern "*.py" \
  --workflow lint_and_test
```

### Resource Access
```bash
# List available local resources
patinox resource list

# Grant filesystem access
patinox resource grant --path /data --mode read

# Execute with container access
patinox agent execute devops --container myapp \
  "Check application logs"
```

### Offline/Online Modes
```bash
# Force offline mode
patinox --offline agent execute analyzer "Process local data"

# Opportunistic mode (default)
patinox agent execute researcher "Find information about Rust"

# Serve local agents to network
patinox serve --protocols mcp,rest --port 8080
```

## Best Practices

1. **Default to Offline**: Assume network isn't available
2. **Explicit Permissions**: Request resource access explicitly
3. **Container Awareness**: Detect and adapt to container environments
4. **Pipeline Friendly**: Support streaming and chunked processing
5. **Scriptable**: Provide consistent, parseable output
6. **Interactive When Needed**: Rich REPL for exploration
7. **Discoverable**: Good help text and completions

## Relationships
- **Parent Nodes:** [elements/architecture_overview.md]
- **Child Nodes:** None
- **Related Nodes:** 
  - [elements/protocol_based_exposure.md] - connects - When online
  - [elements/configuration_strategy.md] - uses - Configuration system
  - [elements/dependency_injection_philosophy.md] - applies - For local services

## Navigation Guidance
- **Access Context:** Reference when implementing CLI interfaces
- **Common Next Steps:** Review configuration strategy or agent conscience
- **Related Tasks:** CLI design, offline operation, container integration
- **Update Patterns:** Update when adding new CLI features or modes

## Metadata
- **Created:** 2025-01-18
- **Last Updated:** 2025-01-18
- **Updated By:** Development Team

## Change History
- 2025-01-18: Initial CLI agent exposure architecture with local-first design