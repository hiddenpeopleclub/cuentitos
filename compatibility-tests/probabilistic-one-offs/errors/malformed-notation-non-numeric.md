# Error: Malformed Notation — Non-Numeric Content

`(fifty%)` has a `%` sign, but its content is not a number at all, so it
cannot be read as a percentage. This is a parse-time error: the story never
starts.

## Script
```cuentitos
This is the story.
(fifty%) A solitary figure sits on the bench.
```

## Input
```input
s
```

## Result
```result
malformed-notation-non-numeric.cuentitos:2: ERROR: Invalid one-off notation: 'fifty%' is neither a percentage ('N%') nor a probability ('0.N').
```
