---
name: code-marshal
description: High-performance, minimalist coding agent driver based on the Vibe engine. Automatically selects the first available agent.
metadata:
  {
    "openclaw": { "emoji": "💂", "requires": { "anyBins": ["code-marshal"] } },
  }
---

# Code-Marshal Skill

Use **code-marshal** to drive heavy-duty coding tasks with normalized log output. It automatically detects and uses the first available coding agent on your system.

## Pattern: background + pty

Always run in the background with PTY enabled to monitor the normalized `[AGENT_EVENT]` stream.

```bash
# Start a coding task (will use the first available agent automatically)
bash pty:true workdir:~/my-project background:true command:"code-marshal 'Refactor the authentication logic to use JWT'"

# Start a task with a specific agent override
bash pty:true workdir:~/my-project background:true command:"code-marshal --agent CURSOR_AGENT 'Explain this complex function'"

# Monitor progress by reading logs
process action:log sessionId:XXX
```

## Options

- `-a, --agent <AGENT>`: Manually specify an agent engine (e.g., `CLAUDE_CODE`, `CURSOR_AGENT`).
- `-l, --list-agents`: List all supported agent engines.
- `-c, --check-installed`: Check which engines are currently installed.

## Discovery

```bash
# List all supported agent engines
code-marshal --list-agents

# Check which ones are currently installed and ready to use
code-marshal --check-installed
```

## Best Practices

1.  **Defaults**: I will automatically pick the first installed engine.
2.  **Overrides**: If you want me to use a specific one, just mention it in your request, and I'll pass the `--agent` flag.
3.  **Memory**: I can remember your preferred agent engine via my memory system.

## Environment Variables

Ensure your provider-specific API keys are set in the environment where `code-marshal` is executed (e.g., `ANTHROPIC_API_KEY`).
