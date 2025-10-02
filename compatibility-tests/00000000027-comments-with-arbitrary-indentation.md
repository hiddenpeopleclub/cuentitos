# Comments with Arbitrary Indentation

Comments can have arbitrary indentation without following hierarchy rules.

## Script
```cuentitos
// Comment at level 0
      // Comment at level 3 (skipping levels)
  // Comment at level 1
Text at level 0
# Section
  Text at level 1
    // Comment at level 2
      // Comment at level 3
  // Comment at level 1
  More text at level 1
```

## Input
```input
s
```

## Result
```result
START
Text at level 0
-> Section
Text at level 1
More text at level 1
END
```
