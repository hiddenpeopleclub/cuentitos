# Named Bucket Branch Participates When Its Req Passes

A named bucket branch (`[(35%) fish_stall] Text`) carries a `req` as a child
line, the same way any other block does. When the draw picks that branch and
its `req` passes, the branch's text shows exactly as a plain branch's would.

## Script
```cuentitos
--- variables
bool market_open = true
---
I approach another stall.
  (40%) The stall is stacked with fresh bread.
  [(35%) fish_stall] The stall smells of fresh fish.
    req market_open
  (25%) The stall sells only trinkets today.
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
END
```
