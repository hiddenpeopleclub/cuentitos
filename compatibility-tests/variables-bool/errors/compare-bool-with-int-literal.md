# Require Error: Comparing a Bool to an Int Literal

A bool variable can only be compared against bool values. Comparing it to an
integer literal is a type error caught at parse time.

## Script
```cuentitos
--- variables
bool door_open = true
---

Line.
  req door_open = 1
```

## Input
```input
s
```

## Result
```result
compare-bool-with-int-literal.cuentitos:6: ERROR: Type mismatch: cannot compare bool 'door_open' with int '1'.
```
