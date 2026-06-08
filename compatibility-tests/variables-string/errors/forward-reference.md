# Error: Forward Reference in a Default

A string default cannot reference a variable declared later in the same
block.

## Script
```cuentitos
--- variables
string a = b
string b = "Aria"
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
