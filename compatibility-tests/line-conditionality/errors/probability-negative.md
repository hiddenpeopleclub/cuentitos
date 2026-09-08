# Error: Negative Probability

Probability notation must fall within `[0, 1]`. `-0.5` is out of range on
the low end. This is a parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(-0.5) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
probability-negative.cuentitos:2: ERROR: Invalid conditionality: '-0.5' is out of range (must be 0-1).
```
