# Escaped Double Quote In a String Literal

A `\"` escape inside a double-quoted literal produces a literal double
quote in the value. `?` re-renders it as `\"`.

## Script
```cuentitos
--- variables
string line = "She said \"hi\""
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
line: "She said \"hi\""
This is the story.
END
```
