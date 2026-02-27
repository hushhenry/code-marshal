---
name: code-marshal
description: High-performance, minimalist coding agent driver. Automatically selects the first available agent.
metadata:
  {
    "openclaw": { "emoji": "💂", "requires": { "anyBins": ["code-marshal"] } }
  }
---

# Code-Marshal Skill

Use `code-marshal` to drive multiple coding agents (Claude Code / Cursor / Codex / OpenCode / Gemini / Qwen, etc.) through one unified CLI.

Key properties:

- Oneshot + follow-up: run a task, then continue multi-turn work via `--follow-up <SESSION_ID>`
- Normalized events: emits a compact event stream (pretty by default; `--json` for machine-readable)
- Automation-first: can be used in non-interactive orchestration flows

## Quick start

```bash
# oneshot
code-marshal -a GEMINI "write a simple html"

# follow-up (multi-turn)
code-marshal -a GEMINI --follow-up <SESSION_ID> "add a button"
```

## Output (important)

- Default output is pretty (human-readable) to reduce token volume.
- Use `--json` to emit machine-readable JSON events.
- Use `--raw` to also include raw child stdout/stderr events.

## Recommended OpenClaw pattern: background

```bash
# Start a coding task
bash workdir:~/my-project background:true command:"code-marshal 'Refactor auth to use JWT'"

# Specify agent
bash workdir:~/my-project background:true command:"code-marshal --agent CURSOR_AGENT 'Explain this function'"

# Follow-up
bash workdir:~/my-project background:true command:"code-marshal --agent GEMINI --follow-up <SESSION_ID> 'add a button'"

# Monitor progress
process action:log sessionId:XXX
```

## CLI options

- `-h, --help`: show help
- `-a, --agent <AGENT>`: specify an agent engine
- `-f, --follow-up <SESSION_ID>`: follow-up prompt in an existing session
- `--json`: emit JSON events instead of pretty output
- `--raw`: also emit raw child stdout/stderr
- `-l, --list-agents`: list supported agent engines
- `-c, --check-installed`: check which engines are installed
