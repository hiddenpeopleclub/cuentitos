# Set Error: String Literal on the RHS of an Enum Set

A double-quoted string literal is a string expression, not a variant
identifier — even when its text matches a declared variant name. Quoting
`"happy"` therefore does not assign the `happy` variant; it is a parse-time
type-mismatch error. The string-literal RHS is echoed in its quoted source
form.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = "happy"
This is the story.
```

## Input
```input
s
```

## Result
```result
set-cross-type-string-literal-rhs.cuentitos:4: ERROR: Type mismatch: 'set' expression for enum mood must be a variant of mood, but '"happy"' is string.
```
