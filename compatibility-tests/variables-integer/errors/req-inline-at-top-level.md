# Require Error: Inline `req` Between Top-Level Text Lines

A `req` placed at the same indentation as surrounding text (rather than
indented as a child of the block it should gate) is a top-level statement,
not a gating child. This produces the same parse-time error as a `req` at
the very top of the script — proving the rule is "`req` must be a child of
the block it gates," not "`req` is allowed as long as something precedes it."

## Script
```cuentitos
--- variables
int x = 5
---

This is text.
req x > 0
More text.
```

## Input
```input
s
```

## Result
```result
req-inline-at-top-level.cuentitos:6: ERROR: Top-level 'req' has no parent block.
```
