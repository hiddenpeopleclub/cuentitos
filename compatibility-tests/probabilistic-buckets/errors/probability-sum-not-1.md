# Error: Probability Bucket Does Not Sum to 1.0

A bucket using probability notation must have branches whose probabilities
sum to exactly 1.0. This bucket sums to 0.8, so the compiler rejects it
before the story ever runs.

## Script
```cuentitos
I visit the street market.
  (0.5) A food vendor calls out to me with a friendly wave.
  (0.3) A street musician plays a quiet, wandering tune.
```

## Input
```input
s
```

## Result
```result
probability-sum-not-1.cuentitos:3: ERROR: Invalid bucket: probability branches must sum to 1.0, but these sum to 0.8.
```
