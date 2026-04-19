# Jump Using Relative Path to Child Section

This test verifies that jumping to a child section using a relative path works correctly.

## Script
```cuentitos
# Parent
-> Child A
  ## Child A
  Text in child A
  ## Child B
  Text in child B
```

## Input
```input
s
```

## Result
```result
START
-> Parent
-> Parent \ Child A
Text in child A
-> Parent \ Child B
Text in child B
END
```
