# Multiple Returns Resume Correct Positions

This test verifies that multiple nested calls resume at the correct positions.

## Script
```cuentitos
# Section A
A1
<-> Section B
A2
<-> Section C
A3

# Section B
B1
<-> Section C
B2

# Section C
C1
```

## Input
```input
s
```

## Result
```result
START
-> Section A
A1
-> Section B
B1
-> Section C
C1
B2
A2
-> Section C
C1
A3
-> Section B
B1
-> Section C
C1
B2
-> Section C
C1
END
```
