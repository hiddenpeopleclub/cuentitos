# Error: Percentage Over 100

Percentage notation must fall within `0-100`. `150%` is out of range on the
high end. This is a parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(150%) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
percentage-out-of-range.cuentitos:2: ERROR: Invalid conditionality: '150%' is out of range (must be 0-100).
```
