# Error: Bool Default Is Not A Bool Literal

A bool variable's default must be `true`, `false`, or a reference to a
previously-declared bool. An integer literal as the default is a type error.

## Script
```cuentitos
--- variables
bool b = 1
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-not-bool-literal.cuentitos:2: ERROR: Type mismatch: default for bool 'b' must be a bool, but '1' is int.
```
