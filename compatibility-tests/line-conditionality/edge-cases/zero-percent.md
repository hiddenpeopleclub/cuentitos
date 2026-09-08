# Edge Case: 0% Is Never Shown

`(0%)` is the percentage-notation lower bound: valid, but the roll never
lands under it, so the line never shows, for any seed.

## Script
```cuentitos
You find a quiet bench.
(0%) A solitary figure sits on the bench, reading.
This serene oasis calms you.
```

## Input
```input
seed 1
s
```

## Result
```result
START
You find a quiet bench.
This serene oasis calms you.
END
```
