# Multiple Jump Commands in One Section

This test verifies that when a section has multiple jump commands, only the first one executes.

## Script
```cuentitos
# Section A
-> Section B
-> Section C

# Section B
Text in B

# Section C
Text in C
```

## Input
```input
s
```

## Result
```result
00000000045-multiple-jumps-in-section.cuentitos:3: WARNING: Unreachable code after section jump
START
-> Section A
-> Section B
Text in B
-> Section C
Text in C
END
```
