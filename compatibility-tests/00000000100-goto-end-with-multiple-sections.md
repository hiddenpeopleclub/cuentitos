# Jump to END with Multiple Sections

This test verifies that -> END skips remaining sections.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B
-> END

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
END
```
