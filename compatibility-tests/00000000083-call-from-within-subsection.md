# Call from Within Subsection

This test verifies that call-and-return works correctly when called from within a subsection.

## Script
```cuentitos
# Parent
In Parent
  ## Child
  In Child
  <-> Target
  Back in Child

# Target
In Target
```

## Input
```input
s
```

## Result
```result
START
-> Parent
In Parent
-> Parent \ Child
In Child
-> Target
In Target
Back in Child
-> Target
In Target
END
```
