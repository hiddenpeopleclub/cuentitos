# Jump to END with Unreachable Code

This test verifies that code after -> END triggers unreachable code warning.

## Script
```cuentitos
# Section A
Text in A
-> END
This is unreachable
```

## Input
```input
s
```

## Result
```result
test.cuentitos:4: WARNING: Unreachable code after section jump
START
-> Section A
Text in A
END
```
