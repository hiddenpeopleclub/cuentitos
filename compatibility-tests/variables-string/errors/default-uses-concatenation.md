# Error: Concatenation in a String Default

String concatenation is not supported in v1. Using `+` in a string
default expression is a parse-time error.

## Script
```cuentitos
--- variables
string greeting = "Hello, " + "world"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-uses-concatenation.cuentitos:2: ERROR: String concatenation is not supported.
```
