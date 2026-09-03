# Error: Percentage Bucket Does Not Sum to 100

A bucket using percentage notation must have branches whose percentages sum
to exactly 100. This bucket sums to 80, so the compiler rejects it before the
story ever runs.

## Script
```cuentitos
I visit the street market.
  (50%) A food vendor calls out to me with a friendly wave.
  (30%) A street musician plays a quiet, wandering tune.
```

## Input
```input
s
```

## Result
```result
percentage-sum-not-100.cuentitos:3: ERROR: Invalid bucket: percentage branches must sum to 100%, but these sum to 80%.
```
