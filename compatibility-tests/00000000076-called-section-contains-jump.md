# Called Section Contains Jump in Middle

This test verifies that if a called section contains a regular jump in the middle, it executes the jump and continues, then returns.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
Start B
-> Section C
After jump in B

# Section C
In C
```

## Input
```input
s
```

## Result
```result
test.cuentitos:9: WARNING: Unreachable code after section jump
START
-> Section A
In A
-> Section B
Start B
-> Section C
Back in A
-> Section B
Start B
-> Section C
In C
END
```
