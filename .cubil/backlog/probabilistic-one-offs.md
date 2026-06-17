---
created: 2026-06-17
---

# Probabilistic One-Offs

## Summary
Support probabilistic one-off text blocks: a line that only appears with a given probability.

Two notations from the reference docs:
- Percentage: `(50%) Text here.` — integer percentages only
- Probability: `(0.5) Text here.` — float in [0, 1] range

A one-off with no probability on siblings is NOT a bucket — it's an independent coin flip for that single line.

## What to do
Run `/grill-with-docs` first to generate a detailed spec.

Then follow TDD: write compatibility tests first (must fail), then implement.

## Reference
- `palabritas.md` (version-0.2) — 'Probabilistic One-Offs' section
- Depends on: `support-for-seeds-in-compatibility-tests`
