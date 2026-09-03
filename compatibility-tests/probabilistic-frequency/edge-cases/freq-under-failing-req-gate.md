# Edge Case: A Failing `req` Excludes a Branch Before Its `freq` Lines Matter

A `req` on a branch is evaluated first and independently of that branch's
`freq` lines, the same way a failing `req` already short-circuits every
other property of the block it guards. Branch B's `req visited = true`
fails (`visited` is `false`), so branch B is excluded before its
`freq mood sad 1000` is ever considered — even though that delta, if it
were applied, would inflate branch B's weight to `1050` and make it
overwhelmingly the most likely branch. Branch A is the only survivor, so it
is chosen regardless of the roll.

## Script
```cuentitos
--- variables
enum mood = happy, sad
bool visited = false
---
set mood = sad
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    req visited = true
    freq mood sad 1000
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
