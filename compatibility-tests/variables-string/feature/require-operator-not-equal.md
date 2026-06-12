# Require: String Not-Equal Operator

A `req` using `!=` passes when the string variable's current value is **not**
equal to the double-quoted literal on the right.

## Script
```cuentitos
--- variables
string name = "Aria"
---

You are not Brenn.
  req name != "Brenn"
You are not Aria.
  req name != "Aria"
```

## Input
```input
s
```

## Result
```result
START
You are not Brenn.
END
```
