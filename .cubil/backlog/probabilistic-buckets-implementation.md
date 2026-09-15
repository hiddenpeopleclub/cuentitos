---
created: 2026-08-29
---

# Probabilistic Buckets — implementation

Implement probabilistic buckets to satisfy
`compatibility-tests/probabilistic-buckets/`.

**Prerequisite:** `probabilistic-buckets-compatibility-tests` done.

## Touch points

- `common/src/block.rs` — represent bucket branches and the bucket grouping.
- `parser/src/parsers/` — form a bucket at an indentation level when there
  are at least two blocks there and every one carries a probability; leave
  every other probabilistic block as a conditional line. Parse named buckets
  `[name]` and `[(N%) name]`.
- Parse-time validation — reject buckets whose probabilities don't sum to
  100% / 1.0, buckets mixing the two notations, named-bucket names that
  aren't snake case, named buckets with a content branch carrying no
  probability, and named buckets with no branches.
- `runtime/src/lib.rs` — weighted draw across bucket branches per visit. Drop
  branches whose `req` fails before drawing, splitting their share equally
  among the survivors.

See ADR 000018 for the formation rule and the equal split.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
