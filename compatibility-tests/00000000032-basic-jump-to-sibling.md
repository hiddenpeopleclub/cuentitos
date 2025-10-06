# Basic Jump to Sibling Section

This test verifies that a basic jump to a sibling section at root level works correctly.

## Script
```cuentitos
# Section A
Text in A
-> Section B

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
START
-> Section A
Text in A
-> Section B
Text in B
-> Section C
Text in C
END
```
