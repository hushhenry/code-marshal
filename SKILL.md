---
name: code-marshal
description: High-performance, minimalist coding agent driver based on the Vibe engine.
metadata:
  {
    "openclaw": { "emoji": "💂", "requires": { "anyBins": ["code-marshal"] } },
  }
---

# Code-Marshal Skill

Use **code-marshal** to drive heavy-duty coding tasks with normalized log output.

## Pattern: background + pty

Always run in the background with PTY enabled to monitor the normalized `[AGENT_EVENT]` stream.

```bash
# Start a coding task in the background
bash pty:true workdir:~/my-project background:true command:"code-marshal 'Refactor the authentication logic to use JWT'"

# Monitor progress by reading logs
process action:log sessionId:XXX

# Code-Marshal will output normalized events:
# [AGENT_EVENT] ToolUse { ... }
# [AGENT_EVENT] AssistantMessage { ... }
```

## Key Features

- **Normalized Output**: Streams structured logs prefixed with `[AGENT_EVENT]`.
- **Auto-Approval**: No human intervention needed during execution.
- **PTY Required**: Ensure `pty:true` is set for proper terminal emulation.

## Environment Variables

Ensure your `ANTHROPIC_API_KEY` or relevant provider keys are set in the environment where `code-marshal` is executed.
