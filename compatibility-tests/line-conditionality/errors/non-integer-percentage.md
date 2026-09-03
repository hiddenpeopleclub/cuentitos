# Error: Non-Integer Percentage

Percentage notation supports whole numbers only. Fractional precision like
`55.5%` is expressible with the probability notation, e.g. `0.555`. This is
a parse-time error: the story never starts.

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
non-integer-percentage.cuentitos:2: ERROR: Invalid conditionality: '50.5%' is not a whole number. Use 0.555 instead.
```
