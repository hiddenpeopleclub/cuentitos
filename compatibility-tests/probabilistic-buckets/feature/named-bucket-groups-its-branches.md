# A Named Bucket Groups Its Branches Under a Name

The plainest form of a named bucket: `[name]` on its own line, with the
branches indented under it. The header carries no probability, so the bucket
is simply drawn when the story reaches it, exactly one branch showing.

The name is snake case. It gives the group an identity to gate with `req` and
to address later; it changes nothing about how the draw works.

The first draw is `0.1916`, which lands in the first slice, `[0, 0.50)`.

## Script
```cuentitos
I step outside to check the sky.
[afternoon_weather]
  (50%) It rains all afternoon.
  (50%) The sun holds until dusk.
I head back in.
```

## Input
```input
seed 6947113883557504045
s
```

## Result
```result
START
I step outside to check the sky.
It rains all afternoon.
I head back in.
END
```
