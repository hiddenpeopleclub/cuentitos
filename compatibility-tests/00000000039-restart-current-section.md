# Restart Current Section Using .

This test verifies that restarting the current section using . works correctly.

## Script
```cuentitos
# Section A
Text in A
-> .
Text after jump
```

## Input
```input
n
n
n
n
n
n
n
n
n
n
q
```

## Result
```result
00000000039-restart-current-section.cuentitos:4: WARNING: Unreachable code after section jump
START
-> Section A
Text in A
-> Section A
Text in A
-> Section A
Text in A
-> Section A
QUIT
```
