# Feature: `freq` Driving a Weight Below Zero Excludes the Branch (High Roll)

Same script as `freq-below-zero-excludes-branch-low-roll.md`, with a
different seed whose roll lands close to `1.0` instead of close to `0.0`.
Branch B is still never drawn: once its weight is 0 or below it is removed
from the bucket before any roll happens, so no roll — high or low — can
select it.

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
seed 10989127208507519847
s
```

## Result
```result
START
Something happens.
Branch A.
END
```
