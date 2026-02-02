# Warning: Multiple Spaces After Arrow

This test verifies that multiple spaces after arrow results in a warning (not an error).

## Script
```cuentitos
# Section A
->  Section B

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
00000000057-malformed-multiple-spaces-after-arrow.cuentitos:2: WARNING: Section name has leading/trailing whitespace: ' Section B'. Trimmed to 'Section B'
START
-> Section A
-> Section B
Text in B
END
```
