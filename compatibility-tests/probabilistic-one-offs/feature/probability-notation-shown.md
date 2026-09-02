# One-Off Probability Notation: Shown

`(0.5) Text` is the same one-off, using the probability notation instead — a
float in `[0, 1]` rather than an integer percentage. With this seed the roll
lands under the threshold, so the line is shown, stripped of its `(0.5)`
prefix.

## Script
```cuentitos
You find a quiet bench.
(0.5) A solitary figure sits on the bench, reading.
This serene oasis calms you.
```

## Input
```input
seed 2951191543015843331
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
