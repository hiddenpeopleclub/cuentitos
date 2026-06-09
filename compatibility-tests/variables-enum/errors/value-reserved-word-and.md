# Error: Enum Value Matches Reserved Keyword `and`

The lowercase logical-operator keywords `and`, `or`, and `not` are reserved by
the `req` boolean grammar and cannot be used as enum values, just as they
cannot be used as variable names. This covers `and`; the `or` and `not`
variants are exercised by their own sibling tests.

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
value-reserved-word-and.cuentitos:2: ERROR: Reserved keyword 'and' cannot be used as an enum value.
```
