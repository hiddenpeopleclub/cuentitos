# One-Off Percentage Notation: Hidden

Same one-off as `percentage-notation-shown`, `(50%) Text`, but with a seed
whose roll lands at or above the threshold. The line is skipped entirely,
leaving no trace in the output — the same way a failing `req` hides its line.

## Script
```cuentitos
You find a quiet bench.
(50%) A solitary figure sits on the bench, reading.
This serene oasis calms you.
```

## Input
```input
seed 7402562259075197047
s
```

## Result
```result
START
You find a quiet bench.
This serene oasis calms you.
END
```
