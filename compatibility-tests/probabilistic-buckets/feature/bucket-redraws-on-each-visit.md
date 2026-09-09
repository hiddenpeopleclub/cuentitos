# A Bucket Draws Again on Every Visit

A bucket draws once per visit. Reaching the same bucket a second time draws
again, and the second draw is independent of the first, so the same bucket
can yield a different branch within a single run.

The two draws are `0.4000` and `0.7448`. The first lands in the rain slice,
`[0, 0.50)`, and the second lands in the sun slice, `[0.50, 1.0)`.

## Script
```cuentitos
# Day
The morning starts.
<-> Weather
I go to work.
<-> Weather
-> END

# Weather
(50%) Rain drums on the roof.
(50%) The sun holds.
```

## Input
```input
seed 1483379947546631278
s
```

## Result
```result
START
-> Day
The morning starts.
-> Weather
Rain drums on the roof.
I go to work.
-> Weather
The sun holds.
END
```
