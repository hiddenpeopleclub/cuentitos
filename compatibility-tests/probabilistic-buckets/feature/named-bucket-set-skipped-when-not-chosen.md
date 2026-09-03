# Named Bucket Branch's Set Is Skipped When Another Branch Is Chosen

The same script as the set-runs-when-chosen test, with a different seed whose
draw picks a different branch. The named branch's text and its `set` never
run, so the later `req`-gated line stays hidden.

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
