# Chain Jumps Between Sections

This test verifies that chaining jumps (A -> B -> C) works correctly.

## Script
```cuentitos
# Section A
-> Section B

# Section B
-> Section C

# Section C
Text in C
```

## Input
```input
s
```

## Result
```result
START
-> Section A
-> Section B
-> Section C
Text in C
END
```
