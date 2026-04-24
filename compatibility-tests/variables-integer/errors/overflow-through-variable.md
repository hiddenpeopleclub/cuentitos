# Error: Integer Overflow Through an Intermediate Variable Reference

Defaults are constant-folded through variable references declared earlier in the same block.
An overflow that occurs after folding through a reference must still be a parse-time error.

## Script
```cuentitos
--- variables
int big = 9223372036854775807
int boom = big + 1
---

This is the story.
```

## Input
```input
s
```

## Result
```result
overflow-through-variable.cuentitos:3: ERROR: Integer overflow in default expression for 'boom'.
```
