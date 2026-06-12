# Multiple Sets in Sequence

Multiple `set` statements in sequence each update the enum variable. The
final value reflects the last assignment.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
---
set mood = happy
set mood = angry
This is the story.
```

## Input
```input
n
?
s
```

## Result
```result
START
This is the story.
mood: angry
END
```
