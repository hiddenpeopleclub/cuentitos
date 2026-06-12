# Set Error: Int Literal on the RHS of an Enum Set

A `set` on an enum variable accepts only a variant of that enum. There is no
implicit int-to-enum coercion, so an integer literal on the RHS is a
parse-time type-mismatch error. This parallels the int-literal rule on the
other typed `set`s (see `variables-string/errors/set-cross-type-int-literal-rhs.md`).

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = 1
This is the story.
```

## Input
```input
s
```

## Result
```result
set-cross-type-int-literal-rhs.cuentitos:4: ERROR: Type mismatch: 'set' expression for enum mood must be a variant of mood, but '1' is int.
```
