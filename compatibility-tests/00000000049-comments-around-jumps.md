# Comments Before and After Jump Commands

This test verifies that comments can appear before and after jump commands.

## Script
```cuentitos
# Section A
// This is a comment before jump
-> Section B
// This is a comment after jump (unreachable)

# Section B
Text in B
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
Text in B
END
```
