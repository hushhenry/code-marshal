# Contributing

Thanks for contributing to Code-Marshal.

## Development setup

Prereqs:
- Rust (stable)

Common commands:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Pull requests

- Keep changes focused and small.
- Add/adjust tests when behavior changes.
- Run formatting + clippy before submitting.

## Reporting issues

Please include:
- OS + architecture
- `code-marshal --help` output
- the exact command you ran
- any relevant logs (use `--json` / `--raw` if needed)
