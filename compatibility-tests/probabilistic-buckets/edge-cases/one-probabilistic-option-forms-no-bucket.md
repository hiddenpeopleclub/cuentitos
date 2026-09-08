# A Lone Probabilistic Option Forms No Bucket

A bucket among options needs at least two probabilistic siblings. With one
probabilistic option and one plain option there is no bucket: the plain
option always appears, and the probabilistic one is rolled on its own. Both
can appear in the same menu, which no bucket reading would allow.

The roll is `0.1916`, under the option's `50%`, so it shows.

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
