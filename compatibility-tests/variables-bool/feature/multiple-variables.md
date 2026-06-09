# Multiple Bool Variables In One Block

A `--- variables` block may declare multiple bool variables; order is preserved
for `?` output, and each prints as `name: true` or `name: false`.

## Script
```cuentitos
--- variables
bool a
bool b = true
bool c = false
bool d = b
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
a: false
b: true
c: false
d: true
This is the story.
END
```
