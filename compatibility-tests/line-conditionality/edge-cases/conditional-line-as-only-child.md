# Edge Case: Conditional Line As The Only Child

A conditional line is defined by having no probabilistic siblings. That
definition holds however many total siblings the block has. A probabilistic
block that happens to be its parent's only child is still a conditional
line: it is rolled independently, using its own chance every time this
point in the story is reached.

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
