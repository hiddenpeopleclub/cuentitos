---
created: 2026-08-29
---

# Probabilistic Frequency Manipulation — compatibility tests

Write the compat tests for the `freq` command **before** the implementation
lands. Sibling task:
`probabilistic-frequency-manipulation-implementation`. TDD: tests first.

## Feature summary

`freq` modifies the effective frequency (weight) of a probabilistic block
when a condition is met. Syntax: `freq <variable> <value> <delta>`.

- Frequencies are weights; the engine normalises across siblings.
- `freq` can take negative values to reduce frequency.
- If the effective frequency drops to 0 or below, the block is excluded
  from the draw.
- Multiple `freq` lines can combine on one block.

## What to cover

- `feature/` — `freq` raising and lowering a branch's weight, exclusion at
  0, multiple `freq` lines combining.
- `errors/` — malformed `freq` line, unknown variable, non-numeric delta.
- `edge-cases/` — negative delta to zero, `freq` combined with a `req` gate.

## Reference

- `palabritas.md` (version-0.2) — frequency notation section.
- Depends on: `probabilistic-buckets` (buckets first).
