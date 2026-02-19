# Code-Marshal 🦞

A minimalist, CLI-first coding agent driver extracted from the Vibe Kanban core. Designed for programmatic control and seamless integration with AI automation systems like OpenClaw.

## Features

- **Minimalist Core**: Stripped-down version of Vibe's execution engine.
- **Auto-Approval**: Injects a `NoopExecutorApprovalService` to run tasks without human intervention.
- **Normalized Logs**: Streams structured agent events to `stdout` with `[AGENT_EVENT]` prefixes.
- **Engine Support**: Powered by `Claude Code` (default) and extensible to other OCI-compatible executors.
- **Non-Blocking**: Built on `tokio` for efficient background processing and log streaming.

## Installation

### Prebuilt binaries (recommended)

Supported platforms (prebuilt):
- Linux: x86_64, aarch64 (ARM64)
- macOS: x86_64, aarch64 (Apple Silicon)

Install the latest release into `~/.local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/hushhenry/code-marshal/master/scripts/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/hushhenry/code-marshal/master/scripts/install.sh | bash -s -- --version v0.1.4
```

### Build from source

```bash
cargo install --path .
```

## Usage

```bash
code-marshal --help

# oneshot
code-marshal -a GEMINI "write a simple html"

# follow-up
code-marshal -a GEMINI --follow-up <SESSION_ID> "add a button"
```

## How it works

Code-Marshal acts as a bridge between high-level AI orchestrators and low-level interactive coding agents. It handles the PTY allocation, protocol parsing, and log normalization, providing a clean stream of events for the orchestrator to monitor.

```text
[SYSTEM] Initializing Code-Marshal...
[SYSTEM] Spawning agent in /path/to/project
[SYSTEM] Task started. Streaming normalized events...
[AGENT_EVENT] ToolUse { tool: "ls", ... }
[AGENT_EVENT] ToolResult { output: "src/ lib.rs ...", ... }
[AGENT_EVENT] AssistantMessage { content: "I've analyzed the files..." }
[SYSTEM] Code-Marshal session concluded.
```

## Development

This project is a fork of [Vibe Kanban](https://github.com/hushhenry/vibe-kanban), specifically focusing on the `executors` and `utils` crates.

### Structure

- `crates/executors`: The heart of the system, managing agent lifecycles.
- `crates/utils`: Shared utilities for logging, process management, and more.
- `src/main.rs`: The minimalist CLI entry point.

## License

MIT / Apache-2.0
