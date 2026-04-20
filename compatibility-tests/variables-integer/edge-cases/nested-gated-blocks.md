# Require Edge Case: Nested Gated Blocks

A child block may carry its own `req` under a parent block that also has a
`req`. The child's `req` is only evaluated when the parent's `req` passes.
A failing `req` at any level skips exactly that block and its descendants —
not its sibling blocks at the same level.

## Script
```cuentitos
--- variables
int outer = 1
int inner = 0
---

Outer passes.
  req outer = 1
  Inner fails.
    req inner = 1
    Deep line hidden.
  Inner passes.
    req inner = 0
    Deep line shown.
Outer fails.
  req outer = 0
  Never shown.
```

## Input
```input
s
```

## Result
```result
START
Outer passes.
Inner passes.
Deep line shown.
END
```
