# Feature: `freq` Lowers a Branch's Weight to Lose the Draw

`freq` can carry a negative delta, shrinking a branch's weight instead of
growing it.

Without the `freq` line, this bucket's weights would be `dad = 30`,
`mom = 10`, `nobody = 60`, giving cumulative ranges `dad [0, 0.30)`,
`mom [0.30, 0.40)`, `nobody [0.40, 1.0)`. The chosen seed rolls `~0.5088`,
which falls inside `nobody`'s range under those baseline weights.

With `freq time_of_day day -50` present and `time_of_day` set to `day`,
nobody's weight becomes `60 - 50 = 10`. The surviving weights are
`dad = 30`, `mom = 10`, `nobody = 10`, total `50`, giving ranges
`dad [0, 0.6)`, `mom [0.6, 0.8)`, `nobody [0.8, 1.0)`. The same roll,
`~0.5088`, now falls inside `dad`'s expanded range, so dad picks up instead
of nobody answering.

## Script
```cuentitos
--- variables
enum time_of_day = day, night
---
set time_of_day = day
I call my parents.
  (30%) The phone rings twice, and dad picks up.
  (10%) The phone rings twice, and mom picks up.
  (60%) The phone rings ten times, nobody is at home.
    freq time_of_day day -50
```

## Input
```input
seed 6267287093519803688
s
```

## Result
```result
START
I call my parents.
The phone rings twice, and dad picks up.
END
```
