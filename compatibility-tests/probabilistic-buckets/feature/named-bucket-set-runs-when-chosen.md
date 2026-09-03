# Named Bucket Branch's Set Runs When Chosen

A named bucket branch can carry a `set` as a child line. When the draw picks
that branch, its text shows and the `set` runs, which a later `req`-gated
line can observe.

## Script
```cuentitos
--- variables
bool met_vendor = false
---
I approach another stall.
  (40%) The stall is stacked with fresh bread.
  [(35%) fish_stall] The stall smells of fresh fish.
    set met_vendor = true
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
The stall smells of fresh fish.
The vendor remembers me now.
END
```
