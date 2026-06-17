---
created: 2026-06-17
---

# Probabilistic Frequency Manipulation

## Summary
Support the `freq` command, which modifies the effective frequency (weight) of a probabilistic block when a condition is met.

Syntax: `freq <variable> <value> <delta>`

Key behaviours:
- Frequencies are weights, not strict probabilities — the engine normalises across siblings
- `freq` can take negative values to reduce frequency
- If effective frequency drops to 0 or below, the block is excluded from the draw
- Can combine multiple `freq` lines on one block

Example:
```
(10) Mom picks up.
  req energy>20
  freq time_of_day night 100
```

## What to do
Run `/grill-with-docs` first to generate a detailed spec.

Then follow TDD: write compatibility tests first (must fail), then implement.

## Reference
- `palabritas.md` (version-0.2) — 'Conditional probability changes' section
- Depends on: `probabilistic-buckets`
