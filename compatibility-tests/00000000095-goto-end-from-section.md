# Jump to END from Within Section

This test verifies that -> END works from within a section.

## Script
```cuentitos
# Section A
Text in A
  ## Subsection
  Text in subsection
  -> END

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
Text in A
-> Section A \ Subsection
Text in subsection
END
```
