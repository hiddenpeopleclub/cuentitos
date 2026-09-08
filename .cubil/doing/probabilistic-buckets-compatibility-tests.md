---
created: 2026-08-29
---

# Probabilistic Buckets — compatibility tests

Write the compat tests for probabilistic buckets **before** the
implementation lands. Sibling task:
`probabilistic-buckets-implementation`. TDD: tests first.

## Feature summary

A bucket is a set of probabilistic sibling blocks where the engine picks
exactly one per visit, respecting the defined probabilities. Any two
probabilistic lines at the same indentation level form a bucket between them.
A probabilistic line with no probabilistic siblings is a conditional line and
is rolled independently.

- Percentage notation: each branch has `(N%)`, must sum to 100.
- Probability notation: each branch has `(0.N)`, must sum to 1.0.
- The compiler rejects invalid sums.

Also includes:
- Named buckets: `[bucket_name]` on its own line, with the bucket's branches
  indented under it. The header carries no text; a probability on it,
  `[(35%) bucket_name]`, makes the group a branch of its parent's bucket. The
  name is snake case, and `req`, `set` and `mod` attach to the group.
- Options as buckets: when all options at an indentation level are
  probabilistic, only one is shown.

## What to cover

- `feature/` — bucket picks one branch across seeds, both notations, the
  chosen branch rendering its own children, a redraw on each visit, a named
  bucket grouping its branches, a named bucket gated by `req`, a failed `req`
  dropping its branch and renormalizing the rest, a named bucket's `set`
  running when its group is chosen and skipped when it is not,
  options-as-buckets.
- `errors/` — percentage sum != 100, probability sum != 1.0, mixed
  notations in one bucket, a named bucket whose name is not snake case, a
  named bucket with a branch that carries no probability, a named bucket with
  no branches at all.
- `edge-cases/` — two-branch bucket, bucket nested under a conditional line,
  plain siblings alongside bucket branches, a lone probabilistic option that
  forms no bucket.

## Reference

- `palabritas.md` (version-0.2) — 'Probabilistic Buckets', 'Named Buckets',
  'Options Bucket' sections.
- Depends on: `line-conditionality` (line conditionality first).
