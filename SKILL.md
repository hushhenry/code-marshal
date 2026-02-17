---
name: code-marshal
description: High-performance, minimalist coding agent driver based on the Vibe engine. Supports switching between different coding agents.
metadata:
  {
    "openclaw": { "emoji": "💂", "requires": { "anyBins": ["code-marshal"] } },
  }
---

# Code-Marshal Skill

Use **code-marshal** to drive heavy-duty coding tasks with normalized log output. You can choose which coding agent to use for the task.

## Pattern: background + pty

Always run in the background with PTY enabled to monitor the normalized `[AGENT_EVENT]` stream.

```bash
# Start a coding task with a specific agent (default: CLAUDE_CODE)
bash pty:true workdir:~/my-project background:true command:"code-marshal --agent CURSOR_AGENT 'Refactor the authentication logic to use JWT'"

# Monitor progress by reading logs
process action:log sessionId:XXX

# Code-Marshal will output normalized events:
# [AGENT_EVENT] ToolUse { ... }
# [AGENT_EVENT] ToolResult { ... }
# [AGENT_EVENT] AssistantMessage { ... }
```

## Options

- `-a, --agent <AGENT>`: Specify the underlying coding agent to drive.
    - Supported agents: `CLAUDE_CODE`, `CURSOR_AGENT`, `CODEX`, `OPENCODE`, `GEMINI`, `QWEN_CODE`.

## Key Features

- **Agent Selection**: Switch between Claude, Cursor, and more using the same interface.
- **Normalized Output**: Streams structured logs prefixed with `[AGENT_EVENT]`.
- **Auto-Approval**: No human intervention needed during execution.
- **PTY Required**: Ensure `pty:true` is set for proper terminal emulation.

## Environment Variables

Ensure your provider-specific API keys are set in the environment where `code-marshal` is executed:
- `ANTHROPIC_API_KEY` (for Claude Code)
- `OPENAI_API_KEY` (for Codex)
- etc.
