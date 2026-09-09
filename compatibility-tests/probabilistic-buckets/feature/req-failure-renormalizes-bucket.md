# A Failed Req Drops Its Branch and Redistributes Its Share

A branch whose `req` fails is out of the running before the draw happens. Its
share is handed to the surviving branches in equal parts, so every survivor
gains the same number of percentage points regardless of how large its own
share already was.

Here `market_open` is false, so the `35%` group is gone. Two branches survive,
so each gains `17.5`: bread goes from `40%` to `57.5%` and trinkets from `25%`
to `42.5%`. The draw is `0.5856`, which lands in the trinket slice under an
equal split and in the bread slice under a proportional one, so the result
tells the two apart.

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
seed 13322711060460899682
s
```

## Result
```result
START
I approach another stall.
The stall sells only trinkets today.
END
```
