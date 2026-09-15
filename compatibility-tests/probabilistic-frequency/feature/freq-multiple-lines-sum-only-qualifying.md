# Feature: Multiple `freq` Lines Sum Only Their Qualifying Deltas

A branch can carry more than one `freq` line. Each is evaluated
independently against the current variable values; only the deltas whose
condition currently holds are added together.

Mom's branch carries two `freq` lines: `freq time_of_day night 40`, whose
condition holds (`time_of_day` is `night`), and `freq is_raining false
1000`, whose condition does not hold (`is_raining` is `true`, not `false`).
Only the qualifying delta is summed, so mom's weight is `10 + 40 = 50`, not
`10 + 40 + 1000 = 1050`.

The surviving weights are `dad = 30`, `mom = 50`, `nobody = 60`, total
`140`, giving ranges `dad [0, 0.2143)`, `mom [0.2143, 0.5714)`,
`nobody [0.5714, 1.0)`. The chosen seed rolls `~0.8959`, landing in
`nobody`'s range. If the non-qualifying `+1000` delta were mistakenly
summed in as well, mom's weight would balloon to `1050` out of a `1140`
total, stretching its range to `[0.0263, 0.9474)` — wide enough to swallow
this same roll and show mom instead. Seeing `nobody` here confirms the
non-qualifying delta was left out.

## Script
```cuentitos
--- variables
enum time_of_day = day, night
bool is_raining = true
---
set time_of_day = night
I call my parents.
  (30%) The phone rings twice, and dad picks up.
  (10%) The phone rings twice, and mom picks up.
    freq time_of_day night 40
    freq is_raining false 1000
  (60%) The phone rings ten times, nobody is at home.
```

## Input
```input
seed 14565638923433801184
s
```

## Result
```result
START
I call my parents.
The phone rings ten times, nobody is at home.
END
```
