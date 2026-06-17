---
created: 2026-06-17
---

# Probabilistic Buckets

## Summary
Support probabilistic buckets: a set of indented blocks where the engine picks exactly one per visit, respecting defined probabilities.

Rules:
- Percentage notation: each branch has `(N%)`, must sum to 100
- Probability notation: each branch has `(0.N)`, must sum to 1.0
- Compiler must reject invalid sums

Also includes:
- **Named buckets**: `[(50%) bucket_name]` — a named branch that can have `req` and `set`
- **Options as buckets**: when all options at an indentation level are probabilistic, only one is shown

## What to do
Run `/grill-with-docs` first to generate a detailed spec.

Then follow TDD: write compatibility tests first (must fail), then implement.

## Reference
- `palabritas.md` (version-0.2) — 'Probabilistic Buckets', 'Named Buckets', 'Options Bucket' sections
- Depends on: `support-for-seeds-in-compatibility-tests`
