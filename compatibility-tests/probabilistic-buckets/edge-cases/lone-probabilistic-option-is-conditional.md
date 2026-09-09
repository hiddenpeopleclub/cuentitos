# A Lone Probabilistic Option Is a Conditional Line

A bucket needs every option at the level to carry a probability. With one
probabilistic option and one plain option there is no bucket: the plain
option appears every time, and the probabilistic one is rolled on its own,
appearing in half of all runs.

The roll is `0.1916`, under the option's `50%`, so it shows. Both options
reach the menu together, which a bucket would never allow.

## Script
```cuentitos
I reach a fork in the road.
  * (50%) Take the hidden trail.
    I went down the trail.
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
  2. Take the main road.
> Selected: Take the hidden trail.
I went down the trail.
END
```
