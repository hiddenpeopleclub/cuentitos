# RESTART Clears Call Stack

This test verifies that -> RESTART clears the call stack, so execution doesn't return from a call.

## Script
```cuentitos
# Section A
Text in A
<-> Section B
This should not execute

# Section B
Text in B
-> RESTART
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
