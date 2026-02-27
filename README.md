# Code-Marshal

A minimalist, CLI-first coding agent driver extracted from the Vibe Kanban core.

Code-Marshal is designed for programmatic control and seamless integration with AI automation/orchestration systems (e.g. OpenClaw).

## Features

- Minimal core: a stripped-down execution engine focused on running coding agents
- Auto-approval: can run tasks without human gating (optional)
- Normalized logs: streams structured events to stdout (pretty or JSON)
- Multi-engine support: Claude Code / Cursor / Codex / OpenCode / Gemini / Qwen (depending on what’s installed)
- Non-blocking: built on Tokio for efficient background processing

## Installation

### Prebuilt binaries (recommended)

Supported platforms:
- Linux: x86_64, aarch64 (ARM64)
- macOS: x86_64, aarch64 (Apple Silicon)

Install the latest release into `~/.local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/WqyJh/code-marshal/master/scripts/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/WqyJh/code-marshal/master/scripts/install.sh | bash -s -- --version v0.1.4
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

### Output modes

- Default: human-friendly pretty output
- `--json`: machine-readable JSON event stream
- `--raw`: also include raw child stdout/stderr

## How it works

Code-Marshal acts as a bridge between high-level orchestrators and low-level interactive coding agents. It handles PTY allocation, protocol parsing, and log normalization, producing a clean event stream that an orchestrator can monitor.

## Development

This project is derived from the Vibe Kanban codebase and focuses on the `executors` and `utils` crates.

Repo structure:
- `crates/executors`: manages agent lifecycles
- `crates/utils`: shared utilities (logging, process management, etc.)
- `src/main.rs`: CLI entry point

### Common commands

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## License

Dual-licensed under MIT OR Apache-2.0.
