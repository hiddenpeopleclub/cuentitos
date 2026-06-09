# Error: Enum Value Matches Reserved Keyword `or`

The lowercase logical-operator keywords `and`, `or`, and `not` are reserved by
the `req` boolean grammar and cannot be used as enum values, just as they
cannot be used as variable names. This is the `or` variant.

## Script
```cuentitos
--- variables
enum mood = happy, or
---

This is the story.
```

## Input
```input
s
```

## Result
```result
value-reserved-word-or.cuentitos:2: ERROR: Reserved keyword 'or' cannot be used as an enum value.
```
