# Call to Child Section

This test verifies that call-and-return works with relative child paths.

## Script
```cuentitos
# Parent
In Parent
<-> Child
Back in Parent
  ## Child
  In Child
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
Back in Parent
-> Parent \ Child
In Child
END
```
