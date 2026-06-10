# Set Error: Compound Assignment on a Bool

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) are arithmetic shortcuts
that have no meaning for a bool variable. Using one on a bool `set` is a
parse-time error.

## Script
```cuentitos
--- variables
bool door_open = false
---
set door_open += true
Hello
```

## Input
```input
s
```

## Result
```result
set-compound-assignment.cuentitos:4: ERROR: Compound assignment ('+=') is not supported for bool variables.
```
