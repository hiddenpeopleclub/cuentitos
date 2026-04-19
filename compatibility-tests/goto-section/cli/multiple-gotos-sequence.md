# Multiple CLI GoTos in Sequence

This test verifies that a user can chain multiple goto commands.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B

# Section C
Text in C
```

## Input
```input
n
n
-> Section B
n
-> Section C
n
-> Section A
s
```

## Result
```result
START
-> Section A
Text in A
-> Section B
Text in B
-> Section C
Text in C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
Text in C
END
```
