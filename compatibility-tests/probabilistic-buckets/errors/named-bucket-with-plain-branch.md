# Error: Named Bucket Has a Branch Without a Probability

Every branch of a named bucket carries a probability. Here the second branch
has none, so the group has no complete distribution to draw from and the
compiler rejects it. Modifier children such as `req`, `set` and `mod` are
exempt; this rule covers content branches.

## Script
```cuentitos
I step outside to check the sky.
[afternoon_weather]
  (50%) It rains all afternoon.
  The sun holds until dusk.
```

## Input
```input
s
```

## Result
```result
named-bucket-with-plain-branch.cuentitos:4: ERROR: Invalid bucket: every branch of a named bucket must carry a probability.
```
