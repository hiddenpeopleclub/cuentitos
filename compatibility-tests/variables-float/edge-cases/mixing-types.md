# Edge Case: Mixing Float With Int in One Block

A single `--- variables` block may declare variables of different types. Each
renders in its own format under `?`: ints as plain integers and floats with at
least one fractional digit. Declaration order is preserved.

## Script
```cuentitos
--- variables
int count = 3
float ratio = 1.5
int total = count + 4
float half = ratio / 2.0
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
count: 3
ratio: 1.5
total: 7
half: 0.75
This is the story.
END
```
