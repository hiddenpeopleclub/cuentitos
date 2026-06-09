# Error: Reserved Keyword `not` as a String Variable Name

`not` is a reserved logical operator keyword from the `req` boolean
grammar. Declaring a string variable with that name is a parse-time error.

## Script
```cuentitos
--- variables
string not = "Aria"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-not.cuentitos:2: ERROR: Reserved keyword 'not' cannot be used as a variable name.
```
