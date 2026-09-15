# A Plain Option Prevents a Bucket

Options follow the same rule as any other block: a bucket forms only when
every option at the level carries a probability. Two probabilistic options
alongside one plain option form no bucket, so each probabilistic option is
rolled on its own and the plain one always appears.

The rolls are `0.1916` and `0.0394`, both under `50%`, so all three options
reach the menu. A bucket would have left at most one of the probabilistic
options standing.

## Script
```cuentitos
I reach a fork in the road.
  * (50%) Take the hidden trail.
    I went down the trail.
  * (50%) Take the tunnel.
    I went through the tunnel.
  * Take the main road.
    I went up the road.
```

## Input
```input
seed 6947113883557504045
1
s
```

## Result
```result
START
I reach a fork in the road.
  1. Take the hidden trail.
  2. Take the tunnel.
  3. Take the main road.
> Selected: Take the hidden trail.
I went down the trail.
END
```
