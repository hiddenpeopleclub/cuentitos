# Error: Empty Enum Value List

An enum declaration must list at least one value. The value list is required;
`enum mood =` with nothing after the `=` is an error.

## Script
```cuentitos
--- variables
enum mood =
---

This is the story.
```

## Input
```input
s
```

## Result
```result
empty-value-list.cuentitos:2: ERROR: Enum 'mood' must declare at least one value.
```
