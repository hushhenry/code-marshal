use std::{collections::HashMap, str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use executors::{
    approvals::NoopExecutorApprovalService,
    env::{ExecutionEnv, RepoContext},
    executors::{BaseCodingAgent, CodingAgent, StandardCodingAgentExecutor},
};
use tokio_stream::StreamExt;
use workspace_utils::{log_msg::LogMsg, msg_store::MsgStore};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    // Common UX: allow `code-marshal help` in addition to --help/-h
    if args.len() == 2 && (args[1] == "help" || args[1] == "--help" || args[1] == "-h") {
        print_usage();
        return Ok(());
    }

    let mut agent_type_str: Option<String> = None;
    let mut follow_up_session_id: Option<String> = None;
    let mut reset_to_message_id: Option<String> = None;
    let mut include_raw_logs = false;
    let mut pretty = false;
    let mut prompt = String::new();

    // Simple arg parsing (intentionally lightweight; clap can be added later)
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
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
                    agent_type_str = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    anyhow::bail!("Missing value for --agent");
                }
            }
            "--follow-up" | "-f" => {
                if i + 1 < args.len() {
                    follow_up_session_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    anyhow::bail!("Missing value for --follow-up <SESSION_ID>");
                }
            }
            "--reset-to" => {
                if i + 1 < args.len() {
                    reset_to_message_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    anyhow::bail!("Missing value for --reset-to <MESSAGE_ID>");
                }
            }
            "--raw" => {
                include_raw_logs = true;
                i += 1;
            }
            "--pretty" => {
                pretty = true;
                i += 1;
            }
            arg if arg.starts_with('-') => {
                anyhow::bail!("Unknown argument: {}", arg);
            }
            arg => {
                // Treat the first positional argument as the prompt (remaining positionals are ignored)
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
        BaseCodingAgent::from_str(&s.to_uppercase()).map_err(|_| {
            anyhow::anyhow!(
                "Unknown agent type: {}. Valid values: CLAUDE_CODE, CURSOR_AGENT, CODEX, OPENCODE, GEMINI, QWEN_CODE, etc.",
                s
            )
        })?
    } else {
        println!("[SYSTEM] No agent specified. Finding first available agent...");
        let available = get_installed_agent_types()?;
        if let Some(first) = available.first() {
            println!("[SYSTEM] Using first available agent: {}", first);
            first.clone()
        } else {
            anyhow::bail!(
                "No coding agents found on system. Please install one (e.g., claude-code, cursor, etc.)"
            );
        }
    };

    println!("[SYSTEM] Initializing Code-Marshal with Agent: {}...", agent_type);

    // 1) Setup executor
    let mut agent = create_agent(agent_type)?;

    // 2) Auto-approval (fully automated)
    let approval_service = Arc::new(NoopExecutorApprovalService::default());
    agent.use_approvals(approval_service);

    // 3) Environment setup
    let current_dir = std::env::current_dir()?;
    let repo_context = RepoContext::new(current_dir.clone(), vec![]);
    let mut env = ExecutionEnv::new(repo_context, false, String::new());

    // Load existing env vars
    let mut vars = HashMap::new();
    for (key, value) in std::env::vars() {
        vars.insert(key, value);
    }
    env.merge(&vars);

    // 4) Spawn agent (initial or follow-up)
    println!("[SYSTEM] Spawning agent in {:?}", current_dir);

    let mut spawned = if let Some(session_id) = follow_up_session_id.as_deref() {
        println!("[SYSTEM] Follow-up session: {}", session_id);
        agent.spawn_follow_up(
            &current_dir,
            &prompt,
            session_id,
            reset_to_message_id.as_deref(),
            &env,
        )
        .await
        .context("Failed to spawn follow-up")?
    } else {
        agent.spawn(&current_dir, &prompt, &env)
            .await
            .context("Failed to spawn agent")?
    };

    // 5) Initialize message store
    //
    // IMPORTANT: in vibe-kanban, the "container" layer streams child stdout/stderr into MsgStore.
    // code-marshal is a CLI, so we must do that wiring here; otherwise normalize_logs has nothing
    // to consume and you won't see SessionId / assistant messages / tool calls.
    let msg_store = Arc::new(MsgStore::new());

    // Wire child stdout/stderr -> MsgStore
    {
        use futures::StreamExt as _;
        use tokio_util::io::ReaderStream;

        if let Some(stdout) = spawned.child.inner().stdout.take() {
            let msg_store_clone = msg_store.clone();
            tokio::spawn(async move {
                let mut stream = ReaderStream::new(stdout);
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            let s = String::from_utf8_lossy(&bytes).into_owned();
                            if !s.is_empty() {
                                msg_store_clone.push_stdout(s);
                            }
                        }
                        Err(e) => {
                            msg_store_clone.push_stderr(format!("[code-marshal] stdout read error: {e}"));
                            break;
                        }
                    }
                }
            });
        }

        if let Some(stderr) = spawned.child.inner().stderr.take() {
            let msg_store_clone = msg_store.clone();
            tokio::spawn(async move {
                let mut stream = ReaderStream::new(stderr);
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            let s = String::from_utf8_lossy(&bytes).into_owned();
                            if !s.is_empty() {
                                msg_store_clone.push_stderr(s);
                            }
                        }
                        Err(e) => {
                            msg_store_clone.push_stderr(format!("[code-marshal] stderr read error: {e}"));
                            break;
                        }
                    }
                }
            });
        }
    }

    // 6) Start log normalization (background)
    {
        let agent_clone = agent.clone();
        let msg_store_clone = msg_store.clone();
        let dir_clone = current_dir.clone();
        tokio::spawn(async move {
            agent_clone.normalize_logs(msg_store_clone, &dir_clone);
        });
    }

    // 7) Stream normalized logs to stdout, and *reliably* terminate when the child exits.
    println!("[SYSTEM] Task started. Streaming normalized events...");

    let mut stream = msg_store.history_plus_stream();
    let mut exit_signal = spawned.exit_signal.take();

    loop {
        tokio::select! {
            // Prefer real process completion over heuristics.
            res = async {
                match &mut exit_signal {
                    Some(rx) => rx.await.ok(),
                    None => None,
                }
            } => {
                // Ensure downstream consumers see a consistent termination marker.
                msg_store.push_finished();
                println!("[SYSTEM] Child process exited: {:?}", res);
                break;
            }
            msg_res = stream.next() => {
                match msg_res {
                    Some(Ok(msg)) => {
                        // By default, print *normalized* events only (JsonPatch/SessionId/etc).
                        // Raw stdout/stderr can be enabled via --raw.
                        let is_raw = matches!(msg, LogMsg::Stdout(_) | LogMsg::Stderr(_));
                        if include_raw_logs || !is_raw {
                            if pretty {
                                pretty_print_logmsg(&msg);
                            } else {
                                let json = serde_json::to_string(&msg)
                                    .unwrap_or_else(|_| format!("{msg:?}"));
                                println!("[AGENT_EVENT] {json}");
                            }
                        }

                        // Surface session id clearly for follow-ups
                        if let LogMsg::SessionId(id) = &msg {
                            println!("[SYSTEM] SessionId: {}", id);
                            println!("[SYSTEM] Follow-up usage: code-marshal -a {} --follow-up {} \"your next prompt\"", agent_type, id);
                        }

                        if matches!(msg, LogMsg::Finished) {
                            println!("[SYSTEM] Finished event received.");
                            break;
                        }
                    }
                    Some(Err(_)) => {
                        // keep going on stream errors
                    }
                    None => {
                        // Stream ended (should be rare); push Finished to close out.
                        msg_store.push_finished();
                        break;
                    }
                }
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
    // Use a single raw string to avoid any weird escaping / parsing issues across toolchains.
    print!(
        r#"Usage: code-marshal [OPTIONS] <PROMPT>

Modes:
  oneshot (default): run a single prompt in a new agent session
  follow-up        : resume/fork an existing session via --follow-up <SESSION_ID>

Options:
  -h, --help                  Show this help
  -a, --agent <AGENT>         Specify the agent to use
                              (Defaults to the first installed agent found)
  -f, --follow-up <SESSION>   Run as follow-up using an existing session id
      --reset-to <MESSAGE_ID> Optional reset point for follow-up (if supported)
      --pretty                Pretty-print normalized events (human readable)
      --raw                   Also emit raw child stdout/stderr events (default: normalized-only)
  -l, --list-agents           List all supported agent types
  -c, --check-installed       Check which agents are installed on the system
"#
    );
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum PatchOp {
    Add,
    Replace,
    Remove,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "SCREAMING_SNAKE_CASE")]
enum PatchValue {
    NormalizedEntry(executors::logs::NormalizedEntry),
    Stdout(String),
    Stderr(String),
    Diff(serde_json::Value),
}

#[derive(serde::Deserialize)]
struct PatchEntry {
    op: PatchOp,
    path: String,
    value: Option<PatchValue>,
}

fn pretty_print_logmsg(msg: &LogMsg) {
    match msg {
        LogMsg::SessionId(id) => {
            println!("[EVENT][session] {id}");
        }
        LogMsg::MessageId(id) => {
            println!("[EVENT][message_id] {id}");
        }
        LogMsg::Finished => {
            println!("[EVENT][finished]");
        }
        LogMsg::Ready => {
            println!("[EVENT][ready]");
        }
        LogMsg::JsonPatch(patch) => {
            // json_patch::Patch is a Vec<PatchOperation> internally, but our patch entries
            // are custom objects (see logs/utils/patch.rs). We parse via serde_json.
            let Ok(v) = serde_json::to_value(patch) else {
                println!("[EVENT][patch] <unserializable>");
                return;
            };
            let Ok(entries) = serde_json::from_value::<Vec<PatchEntry>>(v) else {
                println!("[EVENT][patch] <unparseable>");
                return;
            };

            for e in entries {
                let kind = match e.op {
                    PatchOp::Add => "add",
                    PatchOp::Replace => "replace",
                    PatchOp::Remove => "remove",
                };

                match e.value {
                    Some(PatchValue::NormalizedEntry(ne)) => {
                        use executors::logs::NormalizedEntryType as T;
                        match &ne.entry_type {
                            T::AssistantMessage => {
                                println!("[EVENT][assistant][{kind}] {}", ne.content.trim_end());
                            }
                            T::SystemMessage => {
                                println!("[EVENT][system][{kind}] {}", ne.content.trim_end());
                            }
                            T::Thinking => {
                                println!("[EVENT][thinking][{kind}] {}", ne.content.trim_end());
                            }
                            T::ErrorMessage { .. } => {
                                println!("[EVENT][error][{kind}] {}", ne.content.trim_end());
                            }
                            T::ToolUse { tool_name, status, .. } => {
                                println!("[EVENT][tool][{kind}] {tool_name} ({status:?}) :: {}", ne.content.trim_end());
                            }
                            other => {
                                println!("[EVENT][entry:{other:?}][{kind}] {}", ne.content.trim_end());
                            }
                        }
                    }
                    Some(PatchValue::Stdout(s)) => {
                        println!("[EVENT][stdout][{kind}] {}", s.trim_end());
                    }
                    Some(PatchValue::Stderr(s)) => {
                        println!("[EVENT][stderr][{kind}] {}", s.trim_end());
                    }
                    Some(PatchValue::Diff(_)) => {
                        println!("[EVENT][diff][{kind}] path={} ", e.path);
                    }
                    None => {
                        println!("[EVENT][patch][{kind}] path={} (no value)", e.path);
                    }
                }
            }
        }
        LogMsg::Stdout(s) => {
            println!("[EVENT][stdout] {}", s.trim_end());
        }
        LogMsg::Stderr(s) => {
            println!("[EVENT][stderr] {}", s.trim_end());
        }
    }
}
