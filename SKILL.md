---
name: code-marshal
description: High-performance, minimalist coding agent driver based on the Vibe engine. Automatically selects the first available agent.
metadata:
  {
    "openclaw": { "emoji": "💂", "requires": { "anyBins": ["code-marshal"] } },
  }
---

# Code-Marshal Skill

Use **code-marshal** to drive multiple coding agents (Claude Code / Cursor / Codex / OpenCode / Gemini / Qwen, etc.) through **one unified CLI**.

Key properties:

- **Oneshot + Follow-up**: run a new task, then continue multi-turn work by resuming/forking a previous session.
- **Normalized events**: emits a stream of normalized events (session id, assistant messages, tool calls/results) as machine-readable JSON.
- **Automation-first**: can run with auto-approvals (no human gating).

---

## Quick Start

### Oneshot (default)

```bash
code-marshal -a GEMINI "write a simple html"
```

The output includes a `SessionId` event. Save it for follow-ups.

### Follow-up (multi-turn)

```bash
code-marshal -a GEMINI --follow-up <SESSION_ID> "add a button"
```

---

## Output Format (important)

By default, code-marshal prints **normalized events only**.

- Each event is printed as a single line prefixed with:
  - `[AGENT_EVENT] <json>`
- Event JSON is `LogMsg` serialized (examples):
  - `{ "SessionId": "..." }`
  - `{ "JsonPatch": [...] }` (normalized assistant/tool entries)
  - `{ "Finished": ... }`

### Pretty printing (human readable)

If you want a more readable stream, add `--pretty`:

```bash
code-marshal --pretty -a GEMINI "write a simple html"
```

### Include raw child stdout/stderr (debugging)

Raw logs are off by default; enable with `--raw`:

```bash
code-marshal -a GEMINI --raw "..."
```

---

## Recommended OpenClaw Pattern: background + PTY

Run in the background with PTY enabled to monitor the event stream.

```bash
# Start a coding task (agent auto-pick if --agent is omitted)
bash pty:true workdir:~/my-project background:true command:"code-marshal 'Refactor auth to use JWT'"

# Specify agent
bash pty:true workdir:~/my-project background:true command:"code-marshal --agent CURSOR_AGENT 'Explain this function'"

# Pretty output (human readable)
bash pty:true workdir:~/my-project background:true command:"code-marshal --pretty --agent GEMINI 'write a simple html'"

# Follow-up (multi-turn)
bash pty:true workdir:~/my-project background:true command:"code-marshal --agent GEMINI --follow-up <SESSION_ID> 'add a button'"

# Monitor progress
process action:log sessionId:XXX
```

---

## CLI Options

- `-h, --help`: show help.
- `-a, --agent <AGENT>`: specify an agent engine (e.g. `CLAUDE_CODE`, `CURSOR_AGENT`, `CODEX`, `OPENCODE`, `GEMINI`, `QWEN_CODE`).
- `-f, --follow-up <SESSION_ID>`: run a follow-up prompt in an existing session.
- (removed) `--reset-to`: reset to message id is not reliably available across executors.
- `--pretty`: pretty-print normalized events.
- `--raw`: also emit raw child stdout/stderr events.
- `-l, --list-agents`: list supported agent engines.
- `-c, --check-installed`: check which engines are installed.

---

## Best Practices

1. **Always capture SessionId** from the first run; it is required for follow-ups.
2. If output seems "stuck", try `--raw` to see whether the underlying agent is waiting for auth/IO.
3. Provider API keys must be set in the shell environment (e.g., `ANTHROPIC_API_KEY`, etc.).
