# Error: Trailing Comma In Enum Value List

A trailing comma leaves an empty value at the end of the list. Empty values are
not allowed: every entry in the value list must be a non-empty identifier. The
trailing comma in `happy, sad,` produces an empty third value, which is an
error. (This pins the behavior: a trailing comma is rejected rather than
silently tolerated.)

## Script
```cuentitos
--- variables
enum mood = happy, sad,
---

This is the story.
```

## Input
```input
s
```

## Result
```result
trailing-comma.cuentitos:2: ERROR: Empty enum value in enum 'mood'.
```
