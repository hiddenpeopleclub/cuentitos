# One-Off Percentage Notation: Shown

`(50%) Text` is a one-off: an independent coin flip for that single line,
using the percentage notation (an integer 0-100). With this seed the roll
lands under the threshold, so the line is shown, stripped of its `(50%)`
prefix — the same way a passing `req` shows its line without echoing the
condition.

## Script
```cuentitos
You find a quiet bench.
(50%) A solitary figure sits on the bench, reading.
This serene oasis calms you.
```

## Input
```input
seed 6612366594532847625
s
```

## Result
```result
START
You find a quiet bench.
A solitary figure sits on the bench, reading.
This serene oasis calms you.
END
```
