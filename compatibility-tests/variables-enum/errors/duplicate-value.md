# Error: Duplicate Enum Value

Listing the same value twice within a single enum is an error; values must be
unique inside their own enum.

## Script
```cuentitos
--- variables
enum mood = happy, sad, happy
---

This is the story.
```

## Input
```input
s
```

## Result
```result
duplicate-value.cuentitos:2: ERROR: Duplicate enum value 'happy' in enum 'mood'.
```
