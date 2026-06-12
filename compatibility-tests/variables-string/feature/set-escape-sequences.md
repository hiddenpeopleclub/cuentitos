# Set: Escape Sequences in the RHS Literal

A `set` RHS string literal honors the same escapes as a default literal:
`\n` (newline), `\\` (backslash), and `\"` (double quote). The runtime
stores the unescaped characters, and `?` re-renders them as the
two-character escapes so the output stays on one line.

## Script
```cuentitos
--- variables
string line = "plain"
---
set line = "x\ny\\z\"w"
Hello
```

## Input
```input
n
?
s
```

## Result
```result
START
Hello
line: "x\ny\\z\"w"
END
```
