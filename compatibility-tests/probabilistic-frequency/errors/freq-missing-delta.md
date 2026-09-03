# Error: `freq` Missing the Delta Argument

`freq <variable> <value> <delta>` requires all three arguments. A line that
supplies a variable and a value but no delta is a parse-time error: the
grammar is fully known from the script text alone, so the story never
starts.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = happy
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    freq mood happy
```

## Input
```input
s
```

## Result
```result
freq-missing-delta.cuentitos:8: ERROR: Malformed expression in 'freq': 'mood happy'.
```
