# Edge Case: Conditional Line As The Only Child

A conditional line is any probabilistic block that is not part of a bucket,
and a bucket needs at least two blocks at one indentation level, all of them
probabilistic. A block that is its parent's only child has no second branch
to pair with, so it is a conditional line: it is rolled independently, using
its own chance every time this point in the story is reached.

The minimum of two is what decides this case. Every block at the level does
carry a probability, since there is only the one, so the count is the only
thing separating a conditional line from a one-branch bucket that would owe
`100%`.

## Script
```cuentitos
You find a quiet bench.
  (30%) A solitary figure sits on the bench, reading.
This serene oasis calms you.
```

## Input
```input
seed 206756381263967695
s
```

## Result
```result
START
You find a quiet bench.
A solitary figure sits on the bench, reading.
This serene oasis calms you.
END
```
