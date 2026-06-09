# Error: Invalid Escape Sequence

Only `\"`, `\n`, and `\\` are supported escapes. Any other backslash
sequence inside a string literal is a parse-time error.

## Script
```cuentitos
--- variables
string name = "a\qb"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
invalid-escape.cuentitos:2: ERROR: Invalid escape sequence '\q' in string literal.
```
