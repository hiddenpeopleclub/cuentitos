# Error: Scientific-Notation Float Literal

Float literals must be written in the plain `<digits>.<digits>` form.
Scientific / exponent notation such as `1e3` is not supported and is a
parse-time error.

## Script
```cuentitos
--- variables
float x = 1e3
---

This is the story.
```

## Input
```input
s
```

## Result
```result
invalid-literal-scientific.cuentitos:2: ERROR: Invalid float literal: '1e3'. Float literals must be written as <digits>.<digits> (e.g. '1.5').
```
