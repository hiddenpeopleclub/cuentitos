# Jump Using Relative Path to Sibling Section

This test verifies that jumping to a sibling section using a relative path works correctly.

## Script
```cuentitos
# Root
  ## Section A
  Text in A
  -> Section B
  ## Section B
  Text in B
```

## Input
```input
s
```

## Result
```result
START
-> Root
-> Root \ Section A
Text in A
-> Root \ Section B
Text in B
END
```
