use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;
use executors::executors::claude::ClaudeCode;
use executors::executors::CodingAgentExecutor;
use executors::approvals::NoopExecutorApprovalService;
use executors::MsgStore;
use anyhow::Result;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: code-marshal <prompt>");
        return Ok(());
    }
    let prompt = &args[1];

    println!("[SYSTEM] Initializing Code-Marshal (Claude Code Engine)...");

    // 1. Setup Executor
    let executor = ClaudeCode::default();
    
    // 2. Setup Auto-Approval
    let _approval_service = Arc::new(NoopExecutorApprovalService::default());
    
    // 3. Environment variables
    let mut env = HashMap::new();
    if let Ok(path) = std::env::var("PATH") {
        env.insert("PATH".to_string(), path);
    }

    // 4. Spawn Agent
    let current_dir = std::env::current_dir()?;
    println!("[SYSTEM] Spawning agent in {:?}", current_dir);
    
    let _spawned = executor.spawn(
        &current_dir,
        prompt,
        &env
    ).await?;

    // 5. Initialize Message Store for Normalized Logs
    let msg_store = Arc::new(MsgStore::new());
    
    // 6. Start Log Normalization (Background)
    let executor_clone = executor.clone();
    let msg_store_clone = msg_store.clone();
    let dir_clone = current_dir.clone();
    
    tokio::spawn(async move {
        if let Err(e) = executor_clone.normalize_logs(msg_store_clone, &dir_clone).await {
            eprintln!("[ERROR] Log normalization failed: {:?}", e);
        }
    });

    // 7. Stream Normalized Logs to Stdout
    println!("[SYSTEM] Task started. Streaming normalized events...");
    let mut stream = msg_store.stream();
    while let Some(msg) = stream.next().await {
        // Output normalized log for OpenClaw to consume
        println!("[AGENT_EVENT] {:?}", msg);
        
        // Basic finish detection: If the agent provides a final result or error
        let msg_str = format!("{:?}", msg);
        if msg_str.contains("Finished") || msg_str.contains("Error") {
            println!("[SYSTEM] Task termination signal detected.");
            break;
        }
    }

    println!("[SYSTEM] Code-Marshal session concluded.");
    Ok(())
}
