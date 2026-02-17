use std::sync::Arc;
use tokio_stream::StreamExt;
use executors::executors::{CodingAgent, StandardCodingAgentExecutor, BaseCodingAgent};
use executors::approvals::NoopExecutorApprovalService;
use executors::env::{ExecutionEnv, RepoContext};
use workspace_utils::msg_store::MsgStore;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let mut agent_type_str: Option<String> = None;
    let mut prompt = String::new();

    // Simple arg parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list-agents" | "-l" => {
                list_agents();
                return Ok(());
            }
            "--check-installed" | "-c" => {
                check_installed_agents().await?;
                return Ok(());
            }
            "--agent" | "-a" => {
                if i + 1 < args.len() {
                    agent_type_str = Some(args[i+1].clone());
                    i += 2;
                } else {
                    anyhow::bail!("Missing value for --agent");
                }
            }
            arg if arg.starts_with("-") => {
                anyhow::bail!("Unknown argument: {}", arg);
            }
            arg => {
                prompt = arg.to_string();
                i += 1;
            }
        }
    }

    if prompt.is_empty() {
        print_usage();
        return Ok(());
    }

    // Determine agent type
    let agent_type = if let Some(s) = agent_type_str {
        BaseCodingAgent::from_str(&s.to_uppercase())
            .map_err(|_| anyhow::anyhow!("Unknown agent type: {}. Valid values: CLAUDE_CODE, CURSOR_AGENT, CODEX, OPENCODE, GEMINI, QWEN_CODE, etc.", s))?
    } else {
        // Default logic: find the first installed agent
        println!("[SYSTEM] No agent specified. Finding first available agent...");
        let available = get_installed_agent_types()?;
        if let Some(first) = available.first() {
            println!("[SYSTEM] Using first available agent: {}", first);
            first.clone()
        } else {
            anyhow::bail!("No coding agents found on system. Please install one (e.g., claude-code, cursor, etc.)");
        }
    };

    println!("[SYSTEM] Initializing Code-Marshal with Agent: {}...", agent_type);

    // 1. Setup Executor
    let mut agent = create_agent(agent_type)?;
    
    // 2. Setup Auto-Approval
    let approval_service = Arc::new(NoopExecutorApprovalService::default());
    agent.use_approvals(approval_service);
    
    // 3. Environment setup
    let current_dir = std::env::current_dir()?;
    let repo_context = RepoContext::new(current_dir.clone(), vec![]);
    let mut env = ExecutionEnv::new(repo_context, false, String::new());
    
    // Load existing env vars
    let mut vars = HashMap::new();
    for (key, value) in std::env::vars() {
        vars.insert(key, value);
    }
    env.merge(&vars);

    // 4. Spawn Agent
    println!("[SYSTEM] Spawning agent in {:?}", current_dir);
    
    let _spawned = agent.spawn(
        &current_dir,
        &prompt,
        &env
    ).await.context("Failed to spawn agent")?;

    // 5. Initialize Message Store for Normalized Logs
    let msg_store = Arc::new(MsgStore::new());
    
    // 6. Start Log Normalization (Background)
    let agent_clone = agent.clone();
    let msg_store_clone = msg_store.clone();
    let dir_clone = current_dir.clone();
    
    tokio::spawn(async move {
        agent_clone.normalize_logs(msg_store_clone, &dir_clone);
    });

    // 7. Stream Normalized Logs to Stdout
    println!("[SYSTEM] Task started. Streaming normalized events...");
    let mut stream = msg_store.history_plus_stream();
    while let Some(msg_res) = stream.next().await {
        if let Ok(msg) = msg_res {
            // Output normalized log for OpenClaw to consume
            println!("[AGENT_EVENT] {:?}", msg);
            
            // Basic finish detection: If the agent provides a final result or error
            let msg_str = format!("{:?}", msg);
            if msg_str.contains("Finished") || msg_str.contains("Error") {
                println!("[SYSTEM] Task termination signal detected.");
                break;
            }
        }
    }

    println!("[SYSTEM] Code-Marshal session concluded.");
    Ok(())
}

fn create_agent(agent_type: BaseCodingAgent) -> Result<CodingAgent> {
    let agent_json = "{}";
    match agent_type {
        BaseCodingAgent::ClaudeCode => Ok(serde_json::from_str::<executors::executors::claude::ClaudeCode>(agent_json)?.into()),
        BaseCodingAgent::CursorAgent => Ok(serde_json::from_str::<executors::executors::cursor::CursorAgent>(agent_json)?.into()),
        BaseCodingAgent::Codex => Ok(serde_json::from_str::<executors::executors::codex::Codex>(agent_json)?.into()),
        BaseCodingAgent::Opencode => Ok(serde_json::from_str::<executors::executors::opencode::Opencode>(agent_json)?.into()),
        BaseCodingAgent::Gemini => Ok(serde_json::from_str::<executors::executors::gemini::Gemini>(agent_json)?.into()),
        BaseCodingAgent::QwenCode => Ok(serde_json::from_str::<executors::executors::qwen::QwenCode>(agent_json)?.into()),
        BaseCodingAgent::Amp => Ok(serde_json::from_str::<executors::executors::amp::Amp>(agent_json)?.into()),
        BaseCodingAgent::Copilot => Ok(serde_json::from_str::<executors::executors::copilot::Copilot>(agent_json)?.into()),
        BaseCodingAgent::Droid => Ok(serde_json::from_str::<executors::executors::droid::Droid>(agent_json)?.into()),
    }
}

fn get_installed_agent_types() -> Result<Vec<BaseCodingAgent>> {
    let all_types = vec![
        BaseCodingAgent::ClaudeCode,
        BaseCodingAgent::CursorAgent,
        BaseCodingAgent::Codex,
        BaseCodingAgent::Opencode,
        BaseCodingAgent::Gemini,
        BaseCodingAgent::QwenCode,
        BaseCodingAgent::Amp,
        BaseCodingAgent::Copilot,
        BaseCodingAgent::Droid,
    ];
    
    let mut installed = Vec::new();
    for at in all_types {
        if let Ok(agent) = create_agent(at.clone()) {
            if agent.get_availability_info().is_available() {
                installed.push(at);
            }
        }
    }
    Ok(installed)
}

async fn check_installed_agents() -> Result<()> {
    println!("[SYSTEM] Checking for installed agent binaries...");
    let installed = get_installed_agent_types()?;
    
    let all_types = vec![
        BaseCodingAgent::ClaudeCode,
        BaseCodingAgent::CursorAgent,
        BaseCodingAgent::Codex,
        BaseCodingAgent::Opencode,
        BaseCodingAgent::Gemini,
        BaseCodingAgent::QwenCode,
        BaseCodingAgent::Amp,
        BaseCodingAgent::Copilot,
        BaseCodingAgent::Droid,
    ];

    for at in all_types {
        let status = if installed.contains(&at) { "INSTALLED" } else { "NOT_FOUND" };
        println!("  - {:<15}: {}", at, status);
    }
    Ok(())
}

fn list_agents() {
    println!("[SYSTEM] Supported Agents:");
    println!("  - CLAUDE_CODE  (Anthropic)");
    println!("  - CURSOR_AGENT (Cursor)");
    println!("  - CODEX        (OpenAI)");
    println!("  - OPENCODE     (OpenCode)");
    println!("  - GEMINI       (Google)");
    println!("  - QWEN_CODE    (Alibaba)");
    println!("  - AMP          (Bloop)");
    println!("  - COPILOT      (GitHub)");
    println!("  - DROID        (Droid)");
}

fn print_usage() {
    println!("Usage: code-marshal [OPTIONS] <PROMPT>");
    println!("");
    println!("Options:");
    println!("  -a, --agent <AGENT>     Specify the agent to use");
    println!("                          (Defaults to the first installed agent found)");
    println!("  -l, --list-agents       List all supported agent types");
    println!("  -c, --check-installed   Check which agents are installed on the system");
}
