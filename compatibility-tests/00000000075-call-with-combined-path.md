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
START
-> Root
-> Root \ Child A
In Child A
-> Root \ Child B
In Child B
Back in Child A
-> Root \ Child B
In Child B
END
```
