---
created: 2026-08-29
---

# Function Calling — implementation

Implement function calling and tags to satisfy
`compatibility-tests/function-calling/`.

**Prerequisite:** `function-calling-compatibility-tests` done.

## Touch points

- `common/src/block.rs` — represent function-call and tag blocks.
- `parser/src/parsers/` — recognise backtick lines and `tag` lines.
- `runtime/src/lib.rs` — forward function calls and tags to the game engine
  via the runtime API.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
