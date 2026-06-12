# Set Error: Int Variable on the RHS of an Enum Set

Enum `set` accepts only a variant literal, never a variable reference of a
different type. An integer variable on the RHS is a parse-time type-mismatch
error. As with the other typed `set`s, the offending variable name is echoed
unquoted (a literal would be quoted).

## Script
```cuentitos
--- variables
enum mood = happy, sad
int count = 3
---
set mood = count
This is the story.
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-variable-rhs.cuentitos:5: ERROR: Type mismatch: 'set' expression for enum mood must be a variant of mood, but count is int.
```
