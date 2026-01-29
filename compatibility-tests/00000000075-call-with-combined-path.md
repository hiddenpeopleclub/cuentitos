# Call with Combined Path

This test verifies that call-and-return works with combined paths like .. \ sibling.

## Script
```cuentitos
# Root
  ## Child A
  In Child A
  <-> .. \ Child B
  Back in Child A
  ## Child B
  In Child B
```

## Input
```input
s
```

## Result
```result
00000000075-call-with-combined-path.cuentitos:4: ERROR: Section not found: .. \ Child B
```
