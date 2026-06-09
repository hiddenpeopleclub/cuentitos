# Error: Enum Declaration Without `=`

An enum declaration must use `=` to introduce its value list. Omitting the
`=` is an error.

## Script
```cuentitos
--- variables
enum mood happy, sad
---

This is the story.
```

## Input
```input
s
```

## Result
```result
missing-equals.cuentitos:2: ERROR: Enum 'mood' must declare values with '= value1, value2, ...'.
```
