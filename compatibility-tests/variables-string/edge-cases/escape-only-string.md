# Edge Case: Escape-Only String

A literal whose entire content is a single escape sequence is valid. The
value is one newline character; `?` re-renders it as `"\n"`.

## Script
```cuentitos
--- variables
string newline = "\n"
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
newline: "\n"
This is the story.
END
```
