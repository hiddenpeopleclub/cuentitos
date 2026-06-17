# Error: Seed Zero

Passing `seed 0` must produce a warning because xorshift64 degenerates to an
all-zero sequence when seeded with 0.

## Script
```cuentitos
Hello.
```

## Input
```input
seed 0
```

## Result
```result
seed-zero.cuentitos:0: WARNING: Invalid seed value "0": seed must be a positive integer.
START
```
