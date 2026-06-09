# Default Referencing an Earlier Bool Variable

A bool default may reference a bool variable declared earlier in the same block.

## Script
```cuentitos
--- variables
bool source = true
bool mirror = source
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
source: true
mirror: true
This is the story.
END
```
