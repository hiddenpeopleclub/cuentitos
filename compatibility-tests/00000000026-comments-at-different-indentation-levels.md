# Comments at Different Indentation Levels

Comments at various indentation levels (following normal hierarchy) should be ignored.

## Script
```cuentitos
// Comment at level 0
# First Section
  // Comment at level 1
  Text at level 1
    // Comment at level 2
    Text at level 2
```

## Input
```input
s
```

## Result
```result
START
-> First Section
Text at level 1
Text at level 2
END
```
