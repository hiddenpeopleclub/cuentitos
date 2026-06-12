# Require: Two String Variables Compared with `!=`

The RHS of a string `req` may reference another string variable. `req a != b`
passes when the two variables currently hold different values.

## Script
```cuentitos
--- variables
string a = "Aria"
string b = "Brenn"
string c = "Aria"
---

A differs from B.
  req a != b
A differs from C.
  req a != c
```

## Input
```input
s
```

## Result
```result
START
A differs from B.
END
```
