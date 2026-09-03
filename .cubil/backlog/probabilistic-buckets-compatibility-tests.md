---
created: 2026-08-29
---

# Probabilistic Buckets — compatibility tests

Write the compat tests for probabilistic buckets **before** the
implementation lands. Sibling task:
`probabilistic-buckets-implementation`. TDD: tests first.

## Feature summary

A bucket is a set of indented sibling blocks where the engine picks exactly
one per visit, respecting the defined probabilities.

- Percentage notation: each branch has `(N%)`, must sum to 100.
- Probability notation: each branch has `(0.N)`, must sum to 1.0.
- The compiler rejects invalid sums.

Also includes:
- Named buckets: `[(50%) bucket_name]` — a named branch that can have
  `req` and `set`.
- Options as buckets: when all options at an indentation level are
  probabilistic, only one is shown.

## What to cover

- `feature/` — bucket picks one branch across seeds, both notations, named
  buckets with `req`/`set`, options-as-buckets.
- `errors/` — percentage sum != 100, probability sum != 1.0, mixed
  notations in one bucket.
- `edge-cases/` — two-branch bucket, bucket nested under a conditional line.

## Reference

- `palabritas.md` (version-0.2) — 'Probabilistic Buckets', 'Named Buckets',
  'Options Bucket' sections.
- Depends on: `line-conditionality` (line conditionality first).
