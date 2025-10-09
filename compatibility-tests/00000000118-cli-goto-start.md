# CLI GoTo START

This test verifies that a user can type goto START to restart from beginning.

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
-> START
n
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
