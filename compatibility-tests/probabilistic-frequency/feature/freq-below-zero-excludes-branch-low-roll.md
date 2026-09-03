# Feature: `freq` Driving a Weight Below Zero Excludes the Branch (Low Roll)

When a branch's final weight (base marker plus every qualifying `freq`
delta) ends up at 0 or below, the branch is excluded from the draw
entirely — the remaining branches are renormalized over just themselves.
Companion test `freq-below-zero-excludes-branch-high-roll.md` uses a
different seed with a very different roll to show this holds regardless of
where the roll lands, not just for one lucky draw.

Branch B starts at weight `50` (from `(50%)`). `freq mood sad -80` fires
because `mood` is `sad`, dropping it to `50 - 80 = -30`, well below zero.
Branch A is the only survivor, so it is chosen no matter what the roll is —
here, a roll close to `0.0`.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = sad
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    freq mood sad -80
```

## Input
```input
seed 1
s
```

## Result
```result
START
Something happens.
Branch A.
END
```
