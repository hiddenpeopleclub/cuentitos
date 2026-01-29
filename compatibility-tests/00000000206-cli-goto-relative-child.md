# CLI GoTo with Relative Child Path

This test verifies that a user can use relative paths to jump to child sections.

## Script
```cuentitos
# Parent
Text in parent
  ## Child A
  Text in child A
  ## Child B
  Text in child B
```

## Input
```input
n
n
n
n
-> Child B
s
```

## Result
```result
START
-> Parent
Text in parent
-> Parent \ Child A
Text in child A
-> Parent \ Child B
Text in child B
END
```
