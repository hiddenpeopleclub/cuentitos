# Seed Visible in Debug Output

After setting a seed with `seed N`, `?` shows `seed: N` as the first field.

## Script
```cuentitos
--- variables
int coins = 100
---

Hello.
```

## Input
```input
seed 1000
?
```

## Result
```result
START
seed: 1000
coins: 100
```
