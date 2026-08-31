---
created: 2026-08-29
---

# Probabilistic One-Offs — implementation

Implement probabilistic one-off text blocks to satisfy
`compatibility-tests/probabilistic-one-offs/`.

**Prerequisite:** `probabilistic-one-offs-compatibility-tests` done.

## Touch points

- `common/src/block.rs` — represent a one-off block (probability + string).
- `parser/src/parsers/` — recognise `(N%)` and `(0.N)` line prefixes.
- `runtime/src/lib.rs` — coin flip via `next_float()` on visit; skip the
  block and its subtree when the draw fails.
- `cli/src/main.rs` — render the one-off text when shown.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
