# Error: Enum Value Is Not A Valid Identifier

Enum values follow the same identifier rules as variable names: they must start
with a letter or underscore. A value that starts with a digit is rejected,
mirroring `variables-integer/errors/invalid-identifier.md`.

## Script
```cuentitos
--- variables
enum mood = happy, 2sad
---

This is the story.
```

## Input
```input
s
```

## Result
```result
value-invalid-identifier.cuentitos:2: ERROR: Invalid enum value: '2sad'. Enum values must start with a letter or underscore.
```
