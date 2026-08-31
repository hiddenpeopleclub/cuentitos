---
created: 2026-08-29
---

# Function Calling — compatibility tests

Write the compat tests for function calling and tags **before** the
implementation lands. Sibling task: `function-calling-implementation`. TDD:
tests first.

## Feature summary

Two ways for the script to communicate with the game engine at runtime:

- Functions (backtick notation): a line wrapped in backticks triggers a
  function call. Parameters are passed as a string vector; the engine
  interprets the types.
- Tags: a named marker attached to a block the engine can react to.

Both are forwarded to the game engine via the runtime API — cuentitos does
not interpret them.

## What to cover

- `feature/` — backtick function call with and without parameters, tag on a
  block, both forwarded to the runtime API.
- `errors/` — unbalanced backticks, malformed tag.
- `edge-cases/` — function call as the only child, tag combined with `req`.

## Reference

- Runtime API for forwarding to the game engine.
