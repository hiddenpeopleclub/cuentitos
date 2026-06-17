---
created: 2026-06-17
---

# Docs for How the Engine Reads Lines

## Summary
Write documentation explaining how the cuentitos engine processes script lines: the line-based parsing model, indentation rules, block hierarchy, and how parent-child relationships are established.

Target audience: contributors and people building tooling on top of cuentitos.

## What to do
Run `/grill-with-docs` first to identify what's already documented and what gaps exist.

Cover at minimum:
- How lines map to blocks
- Indentation (2 spaces per level) and parent-child relationships
- How registered block parsers are tried in order
- How strings are stored and referenced by ID
- The `Database` output structure

## Reference
- `CLAUDE.md` — Architecture section
- `parser/src/` source code
- `docs/architecture/` ADRs
