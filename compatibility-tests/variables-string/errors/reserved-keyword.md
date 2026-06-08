# Error: Reserved Keyword as Variable Name

The lowercase logical-operator keywords `and`, `or`, and `not` are
reserved by the `req` boolean grammar and cannot be declared as variable
names, including for string variables.

## Script
```cuentitos
--- variables
string and = "Aria"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword.cuentitos:2: ERROR: Reserved keyword 'and' cannot be used as a variable name.
```
