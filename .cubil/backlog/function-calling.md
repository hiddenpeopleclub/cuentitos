---
created: 2026-06-17
---

# Function Calling

## Summary
Support two ways for the script to communicate with the game engine at runtime:

### Functions (backtick notation)
A line wrapped in backticks triggers a function call to the game engine:
```
\`play_sound alarm\`
\`play_sound alarm 0.3\`
```
Parameters are passed as a string vector. The engine is responsible for interpreting types.

### Tags
A tag is a named marker attached to a block that the engine can react to:
```
* Shelter the dog.
  tag important_decision
```

Both are forwarded to the game engine via the runtime API — cuentitos does not interpret them.

## What to do
Run `/grill-with-docs` first to generate a detailed spec.

Then follow TDD: write compatibility tests first (must fail), then implement.

## Reference
- `palabritas.md` (version-0.2) — 'Functions and tags' section
