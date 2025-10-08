# Call to RESTART with Warning

This test verifies that <-> RESTART produces a warning about not returning.

## Script
```cuentitos
# Section A
Text in A
<-> RESTART
This will not execute
```

## Input
```input
n,n,n,n,n,n,n,n,n,n,q
```

## Result
```result
test.cuentitos:3: WARNING: <-> RESTART will not return (clears state and restarts from beginning)
START
-> Section A
Text in A
START
-> Section A
Text in A
START
-> Section A
Text in A
QUIT
```
