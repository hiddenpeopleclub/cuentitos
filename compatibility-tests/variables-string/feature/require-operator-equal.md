# Require: String Equal Operator

A `req` using `=` compares a string variable against a double-quoted
literal. The comparison passes when the variable's current value equals the
literal exactly.

## Script
```cuentitos
--- variables
string name = "Aria"
---

You greet Aria.
  req name = "Aria"
You greet Brenn.
  req name = "Brenn"
```

## Input
```input
s
```

## Result
```result
START
You greet Aria.
END
```
