# CLI GoTo RESTART

This test verifies that a user can type goto RESTART to reset state and restart.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B
```

## Input
```input
n
n
-> RESTART
n
s
```

## Result
```result
START
-> Section A
Text in A
START
-> Section A
Text in A
-> Section B
Text in B
END
```
