# Require Edge Case: Comparing Against the Empty String

The empty string `""` is a valid string value and a valid comparison RHS. A
variable initialized to `""` passes `req note = ""` and fails any comparison
against a non-empty literal.

## Script
```cuentitos
--- variables
string note = ""
---

Note is empty.
  req note = ""
Note is non-empty.
  req note = "x"
```

## Input
```input
s
```

## Result
```result
START
Note is empty.
END
```
