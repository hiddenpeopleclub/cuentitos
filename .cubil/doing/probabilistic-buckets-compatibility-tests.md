---
created: 2026-08-29
---

# Probabilistic Buckets — compatibility tests

Write the compat tests for probabilistic buckets **before** the
implementation lands. Sibling task:
`probabilistic-buckets-implementation`. TDD: tests first.

## Feature summary

A bucket is a set of probabilistic sibling blocks where the engine picks
exactly one per visit, respecting the defined probabilities.

A bucket forms at an indentation level when there are at least two blocks at
that level and every one of them carries a probability. One plain sibling is
enough to stop a bucket from forming, and so is having only one block at the
level. Every probabilistic block outside a bucket is a conditional line,
rolled independently. This rule is the same for text lines and for options.

- Percentage notation: each branch has `(N%)`, must sum to 100.
- Probability notation: each branch has `(0.N)`, must sum to 1.0.
- The compiler rejects invalid sums.
- A branch whose `req` fails leaves the bucket before the draw. Its share is
  split equally among the surviving branches.

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
  dropping its branch and its share splitting equally, a named bucket's `set`
  running when its group is chosen and skipped when it is not,
  options-as-buckets.
- `errors/` — percentage sum != 100, probability sum != 1.0, mixed
  notations in one bucket, a named bucket whose name is not snake case, a
  named bucket with a branch that carries no probability, a named bucket with
  no branches at all.
- `edge-cases/` — two-branch bucket, bucket nested under a conditional line,
  a plain sibling preventing a bucket among text lines and among options, a
  lone probabilistic option standing as a conditional line.

## Reference

- `palabritas.md` (version-0.2) — 'Probabilistic Buckets', 'Named Buckets',
  'Options Bucket' sections.
- ADR 000018 — bucket formation and the equal split of a gated-out share.
- Depends on: `line-conditionality` (line conditionality first).
