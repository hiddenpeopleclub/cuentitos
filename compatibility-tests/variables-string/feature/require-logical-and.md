# Require: Logical AND over String Comparisons

Two string comparisons may be joined with `and`. The parent block is shown
only when **both** comparisons pass.

## Script
```cuentitos
--- variables
string hero = "Aria"
string villain = "Morgath"
---

Both names match.
  req hero = "Aria" and villain = "Morgath"
One name is wrong.
  req hero = "Aria" and villain = "Brenn"
```

## Input
```input
s
```

## Result
```result
START
Both names match.
END
```
