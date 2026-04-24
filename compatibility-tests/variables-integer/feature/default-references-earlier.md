# Default Referencing an Earlier Variable

A default expression may reference a variable declared earlier in the same block.

## Script
```cuentitos
--- variables
int a = 3
int b = a
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
a: 3
b: 3
This is the story.
END
```
