# Escaped Backslash In a String Literal

A `\\` escape inside a double-quoted literal produces a single backslash
in the value. `?` re-renders it as `\\`.

## Script
```cuentitos
--- variables
string path = "a\\b"
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
path: "a\\b"
This is the story.
END
```
