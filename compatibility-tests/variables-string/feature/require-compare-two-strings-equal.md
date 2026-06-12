# Require: Two String Variables Compared with `=`

The right-hand side of a string `req` may reference another string variable
instead of a literal. `req a = b` passes when both variables currently hold
the same value.

## Script
```cuentitos
--- variables
string a = "Aria"
string b = "Aria"
string c = "Brenn"
---

A matches B.
  req a = b
A matches C.
  req a = c
```

## Input
```input
s
```

## Result
```result
START
A matches B.
END
```
