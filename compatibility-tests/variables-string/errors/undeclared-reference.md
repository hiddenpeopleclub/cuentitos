# Error: Reference to Undeclared Variable

A string default cannot reference a variable that is never declared. This
differs from a forward reference (declared later): the name does not exist
anywhere in the block.

## Script
```cuentitos
--- variables
string a = unknown
---

This is the story.
```

## Input
```input
s
```

## Result
```result
undeclared-reference.cuentitos:2: ERROR: Undefined variable: 'unknown'.
```
