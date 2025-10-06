# Text Before Jump Command

This test verifies that text before a jump command executes normally.

## Script
```cuentitos
# Section A
Text in A before jump
More text in A
-> Section B

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
Text in A before jump
More text in A
-> Section B
Text in B
END
```
