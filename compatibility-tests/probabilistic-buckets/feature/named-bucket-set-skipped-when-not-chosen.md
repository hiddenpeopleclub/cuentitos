# A Named Bucket's Set Is Skipped When Another Branch Is Chosen

The same script as the set-runs-when-chosen test, under a seed whose draw
picks a different branch of the outer bucket. The group is never entered, so
its `set` never runs, its inner bucket never draws, and the later `req`-gated
line stays hidden.

The first draw is `0.1916`, which lands in the bread slice, `[0, 0.40)`. One
draw is spent in the whole run: the group's inner bucket costs nothing when
the group is not chosen.

## Script
```cuentitos
--- variables
bool met_vendor = false
---
I approach another stall.
  (40%) The stall is stacked with fresh bread.
  [(35%) fish_stall]
    set met_vendor = true
    (50%) The stall smells of fresh fish.
    (50%) The stall is piled high with ice.
  (25%) The stall sells only trinkets today.
The vendor remembers me now.
  req met_vendor
```

## Input
```input
seed 6947113883557504045
s
```

## Result
```result
START
I approach another stall.
The stall is stacked with fresh bread.
END
```
