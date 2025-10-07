# Call to END with Warning

This test verifies that <-> END produces a warning about not returning.

## Script
```cuentitos
# Section A
Text in A
<-> END
This will not execute
```

## Input
```input
s
```

## Result
```result
test.cuentitos:3: WARNING: <-> END will not return (just end execution)
START
-> Section A
Text in A
END
```
