# A Failed Req Drops Its Branch and Renormalizes the Bucket

A branch whose `req` fails is out of the running before the draw happens. The
remaining branches share the whole range between them, in proportion to their
declared weights.

Here `market_open` is false, so the `35%` group is gone and the surviving
branches renormalize: bread takes `[0, 0.6154)` and trinkets takes
`[0.6154, 1.0)`. The draw is `0.7199`, which lands in the trinket slice under
renormalization and in the vacated fish slice under a naive reading, so the
result tells the two apart.

## Script
```cuentitos
--- variables
bool market_open = false
---
I approach another stall.
  (40%) The stall is stacked with fresh bread.
  [(35%) fish_stall]
    req market_open
    (50%) The stall smells of fresh fish.
    (50%) The stall is piled high with ice.
  (25%) The stall sells only trinkets today.
```

## Input
```input
seed 9277395066787507847
s
```

## Result
```result
START
I approach another stall.
The stall sells only trinkets today.
END
```
