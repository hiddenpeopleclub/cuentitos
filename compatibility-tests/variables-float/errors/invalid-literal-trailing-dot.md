# Error: Float Literal With a Trailing Dot

A float literal must have digits on both sides of the decimal point. A
trailing dot such as `1.` is not a valid literal and is a parse-time error.

## Script
```cuentitos
--- variables
float x = 1.
---

This is the story.
```

## Input
```input
s
```

## Result
```result
invalid-literal-trailing-dot.cuentitos:2: ERROR: Invalid float literal: '1.'. Float literals must be written as <digits>.<digits> (e.g. '1.5').
```
