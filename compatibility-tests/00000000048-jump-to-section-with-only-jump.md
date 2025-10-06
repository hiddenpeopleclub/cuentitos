# Jump to Section Containing Only a Jump Command

This test verifies that jumping to a section that only contains a jump command works correctly.

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
