# Jump to Nested Section Using Absolute Path

This test verifies that jumping to a nested section using an absolute path works correctly.

## Script
```cuentitos
# Main
Text in main
-> Root \ Sub A

# Root
Text in root
  ## Sub A
  Text in sub A
  ## Sub B
  Text in sub B
```

## Input
```input
s
```

## Result
```result
START
-> Main
Text in main
-> Root \ Sub A
Text in sub A
-> Root \ Sub B
Text in sub B
END
```
