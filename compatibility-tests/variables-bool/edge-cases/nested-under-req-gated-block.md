# Require Edge Case: Bool `req` Nested Under Another Bool-Gated Block

A child block may carry its own bool `req` under a parent block that also has a
bool `req`. The child's `req` is only evaluated when the parent's `req` passes.
A failing `req` at any level skips exactly that block and its descendants — not
its sibling blocks at the same level.

## Script
```cuentitos
--- variables
bool outer = true
bool inner = true
---

Outer passes.
  req outer
  Inner fails.
    req inner = false
    Deep line hidden.
  Inner passes.
    req inner
    Deep line shown.
Outer fails.
  req not outer
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
