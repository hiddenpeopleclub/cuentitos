# Error: Unterminated String Literal

A string literal that opens a double quote but never closes it on the
same line is a parse-time error.

## Script
```cuentitos
--- variables
string name = "Aria
---

This is the story.
```

## Input
```input
s
```

## Result
```result
unterminated-literal.cuentitos:2: ERROR: Unterminated string literal.
```
