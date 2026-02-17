use std::path::PathBuf;
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

    let mut agent_type = BaseCodingAgent::ClaudeCode;
    let mut prompt = String::new();

    // Simple arg parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--agent" | "-a" => {
                if i + 1 < args.len() {
                    agent_type = BaseCodingAgent::from_str(&args[i+1].to_uppercase())
                        .map_err(|_| anyhow::anyhow!("Unknown agent type: {}. Valid values: CLAUDE_CODE, CURSOR_AGENT, CODEX, OPENCODE, GEMINI, QWEN_CODE, etc.", args[i+1]))?;
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

    println!("[SYSTEM] Initializing Code-Marshal with Agent: {}...", agent_type);

    // 1. Setup Executor
    let agent_json = "{}";
    let mut agent: CodingAgent = match agent_type {
        BaseCodingAgent::ClaudeCode => serde_json::from_str::<executors::executors::claude::ClaudeCode>(agent_json)?.into(),
        BaseCodingAgent::CursorAgent => serde_json::from_str::<executors::executors::cursor::CursorAgent>(agent_json)?.into(),
        BaseCodingAgent::Codex => serde_json::from_str::<executors::executors::codex::Codex>(agent_json)?.into(),
        BaseCodingAgent::Opencode => serde_json::from_str::<executors::executors::opencode::Opencode>(agent_json)?.into(),
        BaseCodingAgent::Gemini => serde_json::from_str::<executors::executors::gemini::Gemini>(agent_json)?.into(),
        BaseCodingAgent::QwenCode => serde_json::from_str::<executors::executors::qwen::QwenCode>(agent_json)?.into(),
        BaseCodingAgent::Amp => serde_json::from_str::<executors::executors::amp::Amp>(agent_json)?.into(),
        BaseCodingAgent::Copilot => serde_json::from_str::<executors::executors::copilot::Copilot>(agent_json)?.into(),
        BaseCodingAgent::Droid => serde_json::from_str::<executors::executors::droid::Droid>(agent_json)?.into(),
    };
    
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
    while let Some(msg_result) = stream.next().await {
        if let Ok(msg) = msg_result {
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

fn print_usage() {
    println!("Usage: code-marshal [OPTIONS] <PROMPT>");
    println!("");
    println!("Options:");
    println!("  -a, --agent <AGENT>  Specify the agent to use (default: CLAUDE_CODE)");
    println!("                       Valid agents: CLAUDE_CODE, CURSOR_AGENT, CODEX, OPENCODE, GEMINI, QWEN_CODE");
}
