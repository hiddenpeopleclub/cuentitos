# Error: Percentage Over 100

A one-off percentage must fall within `0-100`. `150%` is out of range. This
is a parse-time error: the story never starts.

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
percentage-out-of-range.cuentitos:2: ERROR: Invalid one-off percentage: '150%' is out of range (must be 0-100).
```
