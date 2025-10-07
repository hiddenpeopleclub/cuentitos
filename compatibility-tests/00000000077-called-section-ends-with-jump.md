# Called Section Ends with Jump

This test verifies that if a called section ends with a regular jump, it executes that section and still returns.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
In B
-> Section C

# Section C
In C
```

## Input
```input
s
```

## Result
```result
START
-> Section A
In A
-> Section B
In B
-> Section C
Back in A
-> Section B
In B
-> Section C
In C
END
```
