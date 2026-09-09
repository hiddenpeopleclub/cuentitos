# Edge Case: A Negative Delta Landing on Exactly Zero Excludes the Branch

The spec excludes a branch whose final weight is "0 or below" — the
boundary value `0` excludes just as much as a negative weight does, not
only weights that overshoot past it. Branch B starts at weight `50`
(`(50%)`) and `freq mood sad -50` fires (`mood` is `sad`), landing its
weight on exactly `0`. Branch A is the only survivor, so it is chosen
regardless of the roll.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = sad
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    freq mood sad -50
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
