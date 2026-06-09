# Edge Case: Negative Zero

IEEE-754 distinguishes `+0.0` from `-0.0`. The engine preserves the sign of
zero, so `-0.0` (whether written directly or produced by `0.0 * -1.0`) renders
as `-0.0`, while a plain `0.0` renders as `0.0`.

## Script
```cuentitos
--- variables
float pos_zero = 0.0
float neg_zero = -0.0
float computed_neg_zero = 0.0 * -1.0
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
pos_zero: 0.0
neg_zero: -0.0
computed_neg_zero: -0.0
This is the story.
END
```
