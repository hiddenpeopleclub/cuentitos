# Edge Case: Whitespace Variants In Value List

Whitespace around the `=`, around commas, and trailing on the line is
insignificant. Each value is trimmed, so `  happy ,sad,  angry  ` declares
exactly three values: `happy`, `sad`, and `angry`. Assigning `angry`
afterward proves the third value parsed correctly despite the irregular
spacing.

## Script
```cuentitos
--- variables
enum mood =   happy ,sad,  angry  
---
set mood = angry
Hello
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
Hello
mood: angry
END
```
