# Named Bucket Gated by a Req Participates When It Passes

A named bucket is a grouping node: `[(35%) fish_stall]` carries no text of its
own, and its content lives in its children, every one of them a probabilistic
branch. A `req` child gates the whole group. When the outer draw picks the
group and its `req` passes, the group's own bucket draws, and that branch's
text is what shows.

With this seed the draws are `0.6091` and `0.4651`. The first lands in the
`fish_stall` slice of the outer bucket, `[0.40, 0.75)`. The second lands in
the first slice of the inner bucket, `[0, 0.50)`.

## Script
```cuentitos
--- variables
bool market_open = true
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
seed 13679095844690443075
s
```

## Result
```result
START
I approach another stall.
The stall smells of fresh fish.
END
```
