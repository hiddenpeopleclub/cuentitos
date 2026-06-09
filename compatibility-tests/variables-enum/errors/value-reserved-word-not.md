# Error: Enum Value Matches Reserved Keyword `not`

The lowercase logical-operator keywords `and`, `or`, and `not` are reserved by
the `req` boolean grammar and cannot be used as enum values, just as they
cannot be used as variable names. This is the `not` variant.

## Script
```cuentitos
--- variables
enum mood = happy, not
---

This is the story.
```

## Input
```input
s
```

## Result
```result
value-reserved-word-not.cuentitos:2: ERROR: Reserved keyword 'not' cannot be used as an enum value.
```
