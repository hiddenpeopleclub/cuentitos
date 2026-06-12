# Require: Logical OR over String Comparisons

Two string comparisons may be joined with `or`. The parent block is shown
when **at least one** comparison passes.

## Script
```cuentitos
--- variables
string hero = "Aria"
---

At least one matches.
  req hero = "Brenn" or hero = "Aria"
Neither matches.
  req hero = "Brenn" or hero = "Cyrus"
```

## Input
```input
s
```

## Result
```result
START
At least one matches.
END
```
