---
created: 2026-08-29
---

# Probabilistic Buckets — implementation

Implement probabilistic buckets to satisfy
`compatibility-tests/probabilistic-buckets/`.

**Prerequisite:** `probabilistic-buckets-compatibility-tests` done.

## Touch points

- `common/src/block.rs` — represent bucket branches and the bucket grouping.
- `parser/src/parsers/` — group probabilistic siblings into a bucket; parse
  named buckets `[(N%) name]`.
- Parse-time validation — reject buckets whose probabilities don't sum to
  100% / 1.0.
- `runtime/src/lib.rs` — weighted draw across bucket branches per visit.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
