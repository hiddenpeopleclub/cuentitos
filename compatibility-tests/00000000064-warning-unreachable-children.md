# Warning: Unreachable Code After Jump - Children

This test verifies that unreachable child blocks after a jump generate a warning.

## Script
```cuentitos
# Section A
Text before
-> Section B
  Indented text after (unreachable)
  More indented text (unreachable)

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
00000000064-warning-unreachable-children.cuentitos:4: WARNING: Unreachable code after section jump
00000000064-warning-unreachable-children.cuentitos:5: WARNING: Unreachable code after section jump
START
-> Section A
Text before
-> Section B
Text in B
END
```
