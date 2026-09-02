# Error: Malformed Notation — Bare Integer

`(50)`, with neither a `%` nor a decimal point, is neither notation: the
percentage notation requires `%` and the probability notation requires a
decimal point. This is a parse-time error: the story never starts.

## Script
```cuentitos
This is the story.
(50) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
malformed-notation-bare-integer.cuentitos:2: ERROR: Invalid one-off notation: '50' is neither a percentage ('N%') nor a probability ('0.N').
```
