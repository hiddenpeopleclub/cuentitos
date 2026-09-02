# Edge Case: One-Off As The Only Child

A one-off is defined by having no probabilistic siblings — not by how many
siblings it has at all. A probabilistic block that happens to be its
parent's only child is still a one-off, rolled independently, not a bucket
of one guaranteed to fire.

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
