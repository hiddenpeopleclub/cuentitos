---
created: 2026-08-29
---

# Line Conditionality — compatibility tests

Write the compat tests for line-conditional text blocks **before** the
implementation lands. Sibling task:
`line-conditionality-implementation`. TDD: tests first.

## Feature summary

A conditional line is a text line that only appears with a given
probability — an independent coin flip for that single line. Two notations:

- Percentage: `(50%) Text here.` — integer percentages only.
- Probability: `(0.5) Text here.` — float in [0, 1].

A conditional line with no probabilistic siblings behaves independently. A
bucket (a separate task) is a set of probabilistic siblings where the
engine picks exactly one.

## What to cover

- `feature/` — conditional line shown and hidden across seeds, both
  notations, conditional line with indented children.
- `errors/` — non-integer percentage, percentage out of range, probability
  out of [0, 1], malformed notation.
- `edge-cases/` — 0% / 100% and 0.0 / 1.0 bounds, conditional line as the
  only child.

## Reference

- `palabritas.md` (version-0.2) — 'Probabilistic One-Offs' section.
- Depends on: `support-for-seeds-in-compatibility-tests` (done).
