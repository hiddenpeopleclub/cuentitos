# Require Error: `req` at the Top Level Has No Parent Block

A `req` only makes sense as a child of the block it gates. A `req` at the top
level of the script (no parent block) is a parse-time error.

## Script
```cuentitos
--- variables
int health = 10
---

req health > 0
```

## Input
```input
s
```

## Result
```result
req-at-top-level.cuentitos:5: ERROR: Top-level 'req' has no parent block.
```
