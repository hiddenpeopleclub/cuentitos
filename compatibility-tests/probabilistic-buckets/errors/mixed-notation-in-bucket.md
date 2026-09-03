# Error: Bucket Mixes Percentage and Probability Notation

A bucket's branches must all use the same notation. Here the first branch
uses percentage notation and the second uses probability notation, which the
compiler rejects regardless of what the two markers would sum to under
either reading.

## Script
```cuentitos
I visit the street market.
  (50%) A food vendor calls out to me with a friendly wave.
  (0.5) A street musician plays a quiet, wandering tune.
```

## Input
```input
s
```

## Result
```result
mixed-notation-in-bucket.cuentitos:3: ERROR: Invalid bucket: mixes percentage and probability notation; a bucket must use one notation throughout.
```
