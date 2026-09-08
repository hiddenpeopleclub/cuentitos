# Error: Probability Over 1

Probability notation must fall within `[0, 1]`. `1.5` is out of range on the
high end. This is a parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(1.5) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
probability-out-of-range.cuentitos:2: ERROR: Invalid conditionality: '1.5' is out of range (must be 0-1).
```
