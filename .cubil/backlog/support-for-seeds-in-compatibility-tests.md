---
created: 2026-06-17
---

# Support for Seeds in Compatibility Tests

## Summary
The compatibility test runner needs to support seeded randomness so that probabilistic features can be tested deterministically.

When a seed is provided, the runtime's RNG must produce consistent output across runs, allowing compat tests to assert exact outcomes for probabilistic blocks (one-offs, buckets, frequency manipulation).

This is a prerequisite for all probabilistic feature tasks.

## What to do
Run `/grill-with-docs` first to generate a detailed spec from the reference documentation and existing test runner code.

Then:
- Add seed support to the runtime RNG
- Add a way to pass a seed to the compat test runner (e.g. a frontmatter field in test .md files or a CLI flag)
- Write compat tests that verify seeded output is stable

## Reference
- `palabritas.md` (version-0.2) — probabilistic sections
- `compat/` test runner code
