# Error: Invalid Seed Value

Passing `seed abc` (a non-numeric value) must produce a warning.

## Script
```cuentitos
Hello.
```

## Input
```input
seed abc
```

## Result
```result
seed-invalid.cuentitos:0: WARNING: Invalid seed value "abc": expected a positive integer.
START
```
