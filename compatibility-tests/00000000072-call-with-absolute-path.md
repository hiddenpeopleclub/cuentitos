# Call with Absolute Path

This test verifies that call-and-return works with absolute paths.

## Script
```cuentitos
# Root
  ## Child A
  In Child A
  <-> Root \ Child B
  Back in Child A
  -> END
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
END
```
