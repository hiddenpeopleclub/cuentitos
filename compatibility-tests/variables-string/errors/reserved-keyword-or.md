# Error: Reserved Keyword `or` as a String Variable Name

`or` is a reserved logical operator keyword from the `req` boolean
grammar. Declaring a string variable with that name is a parse-time error.

## Script
```cuentitos
--- variables
string or = "Aria"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-or.cuentitos:2: ERROR: Reserved keyword 'or' cannot be used as a variable name.
```
