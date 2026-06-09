# Edge Case: Explicit Empty String Default

A string declared with an explicit empty literal `""` is valid and
indistinguishable from one declared without a default.

## Script
```cuentitos
--- variables
string name = ""
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
name: ""
This is the story.
END
```
