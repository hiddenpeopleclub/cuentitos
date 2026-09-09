# A Plain Sibling Prevents a Bucket

A bucket forms at an indentation level when every block at that level carries
a probability. One plain sibling is enough to stop it: the probabilistic
blocks then stand on their own as conditional lines, each rolled
independently.

Here the two `50%` lines share a level with a plain line, so no bucket forms.
The rolls are `0.1916` and `0.0394`, both under `50%`, so both lines show. A
bucket would have picked exactly one of them, so seeing both settles it.

## Script
```cuentitos
I reach a fork in the road.
  (50%) I see a light to the left.
  (50%) I see a red light to the right.
  I can't see anything behind.
```

## Input
```input
seed 6947113883557504045
s
```

## Result
```result
START
I reach a fork in the road.
I see a light to the left.
I see a red light to the right.
I can't see anything behind.
END
```
