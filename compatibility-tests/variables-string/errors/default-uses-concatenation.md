# Error: Concatenation in a String Default

String concatenation is not supported in v1. Using `+` in a string
default expression is a parse-time error. The default-expression parser
does not special-case the `+` between strings; it simply fails to parse
the expression and reports the same `Malformed default expression`
diagnostic used for any other unparseable default, echoing the offending
expression.

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
default-uses-concatenation.cuentitos:2: ERROR: Malformed default expression: '"Hello, " + "world"'.
```
