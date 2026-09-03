# Feature: `freq` Raises a Branch's Weight to Win the Draw

A bucket's branches start at the weight given by their own percentage marker.
`freq <variable> <value> <delta>` adds `<delta>` to a branch's weight whenever
the named variable currently equals `<value>`, before the draw for that
bucket happens.

Without the `freq` line, this bucket's weights would be `dad = 30`,
`mom = 10`, `nobody = 60` (they must sum to `100`, the percentage-notation
requirement), giving cumulative ranges `dad [0, 0.30)`, `mom [0.30, 0.40)`,
`nobody [0.40, 1.0)`. The chosen seed rolls `~0.5013`, which falls inside
`nobody`'s range under those baseline weights.

With `freq time_of_day night 60` present and `time_of_day` set to `night`,
mom's weight becomes `10 + 60 = 70`. The surviving weights are
`dad = 30`, `mom = 70`, `nobody = 60`, total `160`, giving ranges
`dad [0, 0.1875)`, `mom [0.1875, 0.625)`, `nobody [0.625, 1.0)`. The same
roll, `~0.5013`, now falls inside `mom`'s expanded range, so mom picks up
instead of nobody answering.

## Script
```cuentitos
--- variables
enum time_of_day = day, night
---
set time_of_day = night
I call my parents.
  (30%) The phone rings twice, and dad picks up.
  (10%) The phone rings twice, and mom picks up.
    freq time_of_day night 60
  (60%) The phone rings ten times, nobody is at home.
```

## Input
```input
seed 3999955365456625230
s
```

## Result
```result
START
I call my parents.
The phone rings twice, and mom picks up.
END
```
