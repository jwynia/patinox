// A complete, self-contained Rust agent example
// Compile with: rustc main.rs -o my-agent
// Or with Cargo: cargo build --release

use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::process::Command;

// Simple agent configuration
struct AgentConfig {
    name: String,
    description: String,
    system_prompt: String,
    provider: String,
    model: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "agent".to_string(),
            description: "A simple AI agent".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            provider: env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            model: env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
        }
    }
}

// Tool trait for extensibility
trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &str) -> Result<String, Box<dyn std::error::Error>>;
}

// Simple function-based tool
struct FnTool {
    name: String,
    description: String,
    handler: Box<dyn Fn(&str) -> Result<String, Box<dyn std::error::Error>> + Send + Sync>,
}

impl Tool for FnTool {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn execute(&self, args: &str) -> Result<String, Box<dyn std::error::Error>> {
        (self.handler)(args)
    }
}

// Core Agent struct
struct Agent {
    config: AgentConfig,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Agent {
    fn new(config: AgentConfig) -> Self {
        Self {
            config,
            tools: HashMap::new(),
        }
    }
    
    fn add_tool(mut self, tool: Box<dyn Tool>) -> Self {
        self.tools.insert(tool.name().to_string(), tool);
        self
    }
    
    fn add_fn_tool<F>(self, name: &str, description: &str, handler: F) -> Self
    where
        F: Fn(&str) -> Result<String, Box<dyn std::error::Error>> + Send + Sync + 'static,
    {
        let tool = Box::new(FnTool {
            name: name.to_string(),
            description: description.to_string(),
            handler: Box::new(handler),
        });
        self.add_tool(tool)
    }
    
    fn run(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // For this example, we'll use a simple mock LLM or call actual API
        if env::var("MOCK_LLM").is_ok() {
            self.mock_llm_run(input)
        } else {
            self.real_llm_run(input)
        }
    }
    
    fn mock_llm_run(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Simple pattern matching for demonstration
        if input.contains("time") {
            if let Some(tool) = self.tools.get("current_time") {
                return tool.execute("");
            }
        }
        
        if input.contains("uppercase") {
            if let Some(tool) = self.tools.get("uppercase") {
                return tool.execute(input);
            }
        }
        
        if input.contains("count") {
            if let Some(tool) = self.tools.get("word_count") {
                return tool.execute(input);
            }
        }
        
        Ok(format!("Processing: {}", input))
    }
    
    fn real_llm_run(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Build tool descriptions for the LLM
        let tools_desc: Vec<String> = self.tools.values()
            .map(|t| format!("- {}: {}", t.name(), t.description()))
            .collect();
        
        let system_prompt = format!(
            "{}\n\nAvailable tools:\n{}\n\nRespond with either a tool call like 'TOOL:tool_name:args' or a direct answer.",
            self.config.system_prompt,
            tools_desc.join("\n")
        );
        
        // Call the actual LLM based on provider
        match self.config.provider.as_str() {
            "openai" => self.call_openai(&system_prompt, input),
            "anthropic" => self.call_anthropic(&system_prompt, input),
            _ => Ok(format!("Provider {} not implemented. Input: {}", self.config.provider, input))
        }
    }
    
    fn call_openai(&self, system_prompt: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENAI_API_KEY")?;
        
        // Build the request
        let request = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": input}
            ],
            "temperature": 0.7
        });
        
        // Use curl for simplicity (in production, use reqwest)
        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg("https://api.openai.com/v1/chat/completions")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", api_key))
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(request.to_string())
            .output()?;
        
        let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        
        if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            // Check if it's a tool call
            if content.starts_with("TOOL:") {
                let parts: Vec<&str> = content.splitn(3, ':').collect();
                if parts.len() >= 2 {
                    if let Some(tool) = self.tools.get(parts[1]) {
                        let args = if parts.len() > 2 { parts[2] } else { "" };
                        return tool.execute(args);
                    }
                }
            }
            Ok(content.to_string())
        } else {
            Err("Failed to get response from OpenAI".into())
        }
    }
    
    fn call_anthropic(&self, system_prompt: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = env::var("ANTHROPIC_API_KEY")?;
        
        let request = serde_json::json!({
            "model": self.config.model,
            "messages": [{"role": "user", "content": input}],
            "system": system_prompt,
            "max_tokens": 1000
        });
        
        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg("https://api.anthropic.com/v1/messages")
            .arg("-H")
            .arg(format!("x-api-key: {}", api_key))
            .arg("-H")
            .arg("anthropic-version: 2023-06-01")
            .arg("-H")
            .arg("content-type: application/json")
            .arg("-d")
            .arg(request.to_string())
            .output()?;
        
        let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        
        if let Some(content) = response["content"][0]["text"].as_str() {
            // Check if it's a tool call
            if content.starts_with("TOOL:") {
                let parts: Vec<&str> = content.splitn(3, ':').collect();
                if parts.len() >= 2 {
                    if let Some(tool) = self.tools.get(parts[1]) {
                        let args = if parts.len() > 2 { parts[2] } else { "" };
                        return tool.execute(args);
                    }
                }
            }
            Ok(content.to_string())
        } else {
            Err("Failed to get response from Anthropic".into())
        }
    }
    
    fn cli(self) -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = env::args().collect();
        
        // Handle CLI arguments
        if args.len() < 2 {
            eprintln!("Usage: {} <input> [--json] [--quiet]", args[0]);
            eprintln!("       {} --help", args[0]);
            eprintln!("       {} --version", args[0]);
            eprintln!("       {} --tools", args[0]);
            std::process::exit(1);
        }
        
        // Parse flags
        let mut json_output = false;
        let mut quiet = false;
        let mut input = String::new();
        
        for arg in &args[1..] {
            match arg.as_str() {
                "--help" | "-h" => {
                    println!("{}: {}", self.config.name, self.config.description);
                    println!("\nUsage: {} <input> [options]", args[0]);
                    println!("\nOptions:");
                    println!("  --json     Output as JSON");
                    println!("  --quiet    Minimal output");
                    println!("  --tools    List available tools");
                    println!("  --help     Show this help message");
                    println!("  --version  Show version");
                    println!("\nEnvironment variables:");
                    println!("  LLM_PROVIDER  LLM provider (openai, anthropic)");
                    println!("  LLM_MODEL     Model name");
                    println!("  MOCK_LLM      Use mock LLM for testing");
                    return Ok(());
                }
                "--version" | "-v" => {
                    println!("{} v0.1.0", self.config.name);
                    return Ok(());
                }
                "--tools" => {
                    println!("Available tools:");
                    for tool in self.tools.values() {
                        println!("  {}: {}", tool.name(), tool.description());
                    }
                    return Ok(());
                }
                "--json" | "-j" => json_output = true,
                "--quiet" | "-q" => quiet = true,
                _ if !arg.starts_with("--") => {
                    if !input.is_empty() {
                        input.push(' ');
                    }
                    input.push_str(arg);
                }
                _ => {
                    eprintln!("Unknown option: {}", arg);
                    std::process::exit(1);
                }
            }
        }
        
        // Check for pipe input
        if input.is_empty() {
            let mut buffer = String::new();
            if let Ok(_) = io::stdin().read_line(&mut buffer) {
                input = buffer.trim().to_string();
            }
        }
        
        if input.is_empty() {
            eprintln!("Error: No input provided");
            std::process::exit(1);
        }
        
        // Run the agent
        match self.run(&input) {
            Ok(output) => {
                if json_output {
                    let json = serde_json::json!({
                        "input": input,
                        "output": output,
                        "success": true
                    });
                    println!("{}", json);
                } else if !quiet {
                    println!("{}", output);
                }
                Ok(())
            }
            Err(e) => {
                if json_output {
                    let json = serde_json::json!({
                        "input": input,
                        "error": e.to_string(),
                        "success": false
                    });
                    eprintln!("{}", json);
                } else {
                    eprintln!("Error: {}", e);
                }
                std::process::exit(1);
            }
        }
    }
}

// Helper to create agents easily
fn create_agent(name: &str) -> Agent {
    Agent::new(AgentConfig {
        name: name.to_string(),
        ..Default::default()
    })
}

// Include serde_json for JSON handling
mod serde_json {
    use std::collections::HashMap;
    
    #[derive(Debug)]
    pub enum Value {
        String(String),
        Number(f64),
        Bool(bool),
        Array(Vec<Value>),
        Object(HashMap<String, Value>),
        Null,
    }
    
    impl Value {
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s),
                _ => None,
            }
        }
    }
    
    // Minimal JSON macro for the example
    #[macro_export]
    macro_rules! json {
        ({ $($key:literal : $value:expr),* }) => {{
            format!("{{ {} }}", 
                vec![$(format!(r#""{}":{}"#, $key, 
                    serde_json::json_value!($value))),*].join(","))
        }};
        ([ $($value:expr),* ]) => {{
            format!("[{}]", vec![$(serde_json::json_value!($value)),*].join(","))
        }};
        ($value:expr) => {{
            serde_json::json_value!($value)
        }};
    }
    
    #[macro_export]
    macro_rules! json_value {
        ($value:literal) => {{
            format!(r#""{}""#, $value)
        }};
        ($value:expr) => {{
            $value.to_string()
        }};
    }
    
    pub fn from_slice(bytes: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
        // Simplified JSON parsing for the example
        let s = String::from_utf8_lossy(bytes);
        Ok(Value::String(s.to_string()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent with some example tools
    let agent = create_agent("example-agent")
        .add_fn_tool("current_time", "Get the current time", |_| {
            use std::time::SystemTime;
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            Ok(format!("Current Unix timestamp: {}", now))
        })
        .add_fn_tool("uppercase", "Convert text to uppercase", |args| {
            Ok(args.to_uppercase())
        })
        .add_fn_tool("word_count", "Count words in text", |args| {
            let count = args.split_whitespace().count();
            Ok(format!("Word count: {}", count))
        })
        .add_fn_tool("reverse", "Reverse text", |args| {
            Ok(args.chars().rev().collect())
        })
        .add_fn_tool("read_file", "Read a file", |path| {
            use std::fs;
            fs::read_to_string(path)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .add_fn_tool("run_command", "Execute a shell command", |cmd| {
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        });
    
    // Run the CLI
    agent.cli()
}

// Compile and run examples:
// 
// 1. Basic compilation:
//    rustc main.rs -o my-agent
//
// 2. With mock LLM (no API needed):
//    MOCK_LLM=1 ./my-agent "what time is it"
//
// 3. With OpenAI:
//    OPENAI_API_KEY=sk-... ./my-agent "explain quantum computing"
//
// 4. With Anthropic:  
//    LLM_PROVIDER=anthropic ANTHROPIC_API_KEY=sk-ant-... ./my-agent "write a haiku"
//
// 5. List tools:
//    ./my-agent --tools
//
// 6. Pipe input:
//    echo "make this uppercase" | ./my-agent
//
// 7. JSON output:
//    ./my-agent "count the words here" --json
//