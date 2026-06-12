# Set: Variant Literal Updates the Enum

A `set <var> = <variant>` statement assigns one of the enum's declared
variants to the variable. The right-hand side is a bare variant identifier
(enum `set` accepts only a variant literal — there is no enum-to-enum copy).
A subsequent `?` reflects the new value rendered as the bare variant name,
unquoted.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
---
set mood = sad
This is the story.
```

## Input
```input
n
?
s
```

## Result
```result
START
This is the story.
mood: sad
END
```
