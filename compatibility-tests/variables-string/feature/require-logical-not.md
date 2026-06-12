# Require: Logical NOT over a String Comparison

A `req` using `not` inverts the truth value of its string comparison
operand. The parent block is shown only when the negated expression is true.

## Script
```cuentitos
--- variables
string door = "locked"
---

Door is not open.
  req not door = "open"
Door is not locked.
  req not door = "locked"
```

## Input
```input
s
```

## Result
```result
START
Door is not open.
END
```
