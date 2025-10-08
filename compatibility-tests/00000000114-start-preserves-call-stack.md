# START Clears Call Stack

This test verifies that -> START clears the call stack (doesn't return from calls).

## Script
```cuentitos
# Section A
Text in A
<-> Section B
This should not execute

# Section B
Text in B
-> START
```

## Input
```input
n,n,n,n,n,n,n,n,q
```

## Result
```result
START
-> Section A
Text in A
-> Section B
Text in B
START
-> Section A
QUIT
```
