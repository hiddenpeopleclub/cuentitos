# Percentage Bucket Picks the First Branch

A bucket is a set of probabilistic sibling blocks where the engine picks exactly
one per visit. This test uses a three-branch percentage bucket (`50%`/`30%`/`20%`)
with a seed whose draw lands in the first branch's slice, proving that branch's
text shows while the other two leave no trace at all.

## Script
```cuentitos
I visit the street market.
  (50%) A food vendor calls out to me with a friendly wave.
  (30%) A street musician plays a quiet, wandering tune.
  (20%) A vendor argues loudly with a customer nearby.
```

## Input
```input
seed 6947113883557504045
s
```

## Result
```result
START
I visit the street market.
A food vendor calls out to me with a friendly wave.
END
```
