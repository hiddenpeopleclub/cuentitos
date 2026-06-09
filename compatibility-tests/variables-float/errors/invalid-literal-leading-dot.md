# Error: Float Literal With a Leading Dot

A float literal must have digits on both sides of the decimal point. A bare
leading dot such as `.5` is not a valid literal and is a parse-time error.

## Script
```cuentitos
--- variables
float x = .5
---

This is the story.
```

## Input
```input
s
```

## Result
```result
invalid-literal-leading-dot.cuentitos:2: ERROR: Invalid float literal: '.5'. Float literals must be written as <digits>.<digits> (e.g. '1.5').
```
