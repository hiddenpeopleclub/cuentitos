# Error: Enum Value Matches Reserved Keyword

The lowercase logical-operator keywords `and`, `or`, and `not` are reserved by
the `req` boolean grammar and cannot be used as enum values, just as they
cannot be used as variable names.

## Script
```cuentitos
--- variables
enum mood = happy, and
---

This is the story.
```

## Input
```input
s
```

## Result
```result
value-reserved-word.cuentitos:2: ERROR: Reserved keyword 'and' cannot be used as an enum value.
```
