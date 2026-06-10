# Error: Forward Reference In A Bool Default

A bool default cannot reference a variable declared later in the same block.

## Script
```cuentitos
--- variables
bool a = b
bool b = true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
forward-reference.cuentitos:2: ERROR: Forward reference: variable 'b' referenced before declaration.
```
