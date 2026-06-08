# Edge Case: Mixing String With Other Types

A `--- variables` block may declare string variables alongside int, bool,
and float variables. Declaration order is preserved for `?`, and each
type renders in its own format (strings double-quoted, others bare).

## Script
```cuentitos
--- variables
int count = 7
bool ready = true
float ratio = 1.5
string name = "Aria"
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
count: 7
ready: true
ratio: 1.5
name: "Aria"
This is the story.
END
```
