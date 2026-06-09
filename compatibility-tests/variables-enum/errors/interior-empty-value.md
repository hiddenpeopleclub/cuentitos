# Error: Empty Value Between Commas In Enum Value List

Two consecutive commas (with only whitespace between them) leave an empty value
in the middle of the list. Empty values are not allowed: every entry must be a
non-empty identifier. The `, ,` in `happy, , sad` produces an empty interior
value, which is an error — the same rule as the trailing-comma case, just in the
middle of the list rather than at the end.

## Script
```cuentitos
--- variables
enum mood = happy, , sad
---

This is the story.
```

## Input
```input
s
```

## Result
```result
interior-empty-value.cuentitos:2: ERROR: Empty enum value in enum 'mood'.
```
