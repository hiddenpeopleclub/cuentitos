# Call to START with Warning

This test verifies that <-> START produces a warning about not returning.

## Script
```cuentitos
# Section A
Text in A
<-> START
This will not execute
```

## Input
```input
n,n,n,n,n,n,n,n,n,n,q
```

## Result
```result
test.cuentitos:3: WARNING: <-> START will not return (restarts from beginning)
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
