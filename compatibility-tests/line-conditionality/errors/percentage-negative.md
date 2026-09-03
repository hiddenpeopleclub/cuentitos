# Error: Negative Percentage

Percentage notation must fall within `0-100`. `-10%` is out of range on the
low end. This is a parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(-10%) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
percentage-negative.cuentitos:2: ERROR: Invalid conditionality: '-10%' is out of range (must be 0-100).
```
