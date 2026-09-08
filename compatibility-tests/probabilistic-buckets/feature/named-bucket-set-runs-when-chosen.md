# A Named Bucket's Set Runs When the Group Is Chosen

A named bucket can carry a `set` alongside its branches. The `set` belongs to
the group, so it runs once when the outer draw picks the group, before the
group's own bucket draws. A later `req`-gated line observes it.

With this seed the draws are `0.6100` and `0.7217`. The first picks
`fish_stall` out of the outer bucket, `[0.40, 0.75)`. The second lands in the
second slice of the inner bucket, `[0.50, 1.0)`.

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
seed 8474225571399870507
s
```

## Result
```result
START
I approach another stall.
The stall is piled high with ice.
The vendor remembers me now.
END
```
