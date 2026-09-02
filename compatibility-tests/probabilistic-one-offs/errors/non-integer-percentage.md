# Error: Non-Integer Percentage

Percentage notation only supports integers — `55.5%` is not allowed, use the
probability notation (`0.555`) for that precision instead. This is a
parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(50.5%) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
non-integer-percentage.cuentitos:2: ERROR: Invalid one-off percentage: '50.5%' is not a whole number.
```
