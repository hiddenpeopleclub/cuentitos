---
created: 2026-08-29
---

# Probabilistic Frequency Manipulation — implementation

Implement the `freq` command to satisfy
`compatibility-tests/probabilistic-frequency-manipulation/`.

**Prerequisite:**
`probabilistic-frequency-manipulation-compatibility-tests` done.

## Touch points

- `parser/src/parsers/` — parse `freq <variable> <value> <delta>` lines.
- `common/src/block.rs` — attach frequency modifiers to a block.
- `runtime/src/lib.rs` — apply `freq` deltas to a branch's weight before the
  bucket draw; exclude branches at weight <= 0.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
