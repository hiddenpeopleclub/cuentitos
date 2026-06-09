# Edge Case: Bool And Int Variables Interleaved

When bool and int declarations are interleaved in the same `--- variables`
block, `?` should print every variable in declaration order, each with its
per-type format.

## Script
```cuentitos
--- variables
int a = 1
bool b = true
int c = 2
bool d = false
int e
bool f
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
a: 1
b: true
c: 2
d: false
e: 0
f: false
This is the story.
END
```
