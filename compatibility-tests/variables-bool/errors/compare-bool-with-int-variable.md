# Require Error: Comparing a Bool to an Int Variable

A bool `req` whose RHS references a variable of a different type is a type
error. Here `health` is an int, so it cannot be compared against the bool
`door_open`.

## Script
```cuentitos
--- variables
bool door_open = true
int health = 10
---

Line.
  req door_open = health
```

## Input
```input
s
```

## Result
```result
compare-bool-with-int-variable.cuentitos:7: ERROR: Type mismatch: cannot compare bool 'door_open' with int 'health'.
```
