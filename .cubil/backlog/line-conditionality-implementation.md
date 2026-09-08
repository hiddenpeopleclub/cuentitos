---
created: 2026-08-29
---

# Line Conditionality — implementation

Implement line-conditional text blocks to satisfy
`compatibility-tests/line-conditionality/`.

**Prerequisite:** `line-conditionality-compatibility-tests` done.

## Touch points

- `common/src/block.rs` — represent a conditional-line block (probability +
  string).
- `parser/src/parsers/` — recognise `(N%)` and `(0.N)` line prefixes.
- `runtime/src/lib.rs` — coin flip via `next_float()` on visit; skip the
  block and its subtree when the draw fails.
- `cli/src/main.rs` — render the conditional-line text when shown.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
