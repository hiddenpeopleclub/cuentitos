# Escaped Newline In a String Literal

A `\n` escape inside a double-quoted literal produces a newline character
in the value. `?` re-renders it as the two-character escape `\n` so the
output stays on one line.

## Script
```cuentitos
--- variables
string line = "a\nb"
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
line: "a\nb"
This is the story.
END
```
