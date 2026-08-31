---
created: 2026-08-29
---

# Probabilistic One-Offs — compatibility tests

Write the compat tests for probabilistic one-off text blocks **before** the
implementation lands. Sibling task:
`probabilistic-one-offs-implementation`. TDD: tests first.

## Feature summary

A one-off is a text line that only appears with a given probability — an
independent coin flip for that single line. Two notations:

- Percentage: `(50%) Text here.` — integer percentages only.
- Probability: `(0.5) Text here.` — float in [0, 1].

A one-off with no probability on its siblings is a one-off. A bucket (a
separate task) is a set of siblings where the engine picks exactly one.

## What to cover

- `feature/` — one-off shown and hidden across seeds, both notations,
  one-off with indented children.
- `errors/` — non-integer percentage, percentage out of range, probability
  out of [0, 1], malformed notation.
- `edge-cases/` — 0% / 100% and 0.0 / 1.0 bounds, one-off as the only child.

## Reference

- `palabritas.md` (version-0.2) — 'Probabilistic One-Offs' section.
- Depends on: `support-for-seeds-in-compatibility-tests` (done).
