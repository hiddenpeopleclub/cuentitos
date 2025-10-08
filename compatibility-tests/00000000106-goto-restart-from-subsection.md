# Jump to RESTART from Within Subsection

This test verifies that -> RESTART works from within nested sections.

## Script
```cuentitos
# Section A
Text in A
  ## Subsection
  Text in subsection
  -> RESTART

# Section B
Text in B
```

## Input
```input
n,n,n,n,n,n,n,q
```

## Result
```result
START
-> Section A
Text in A
-> Section A \ Subsection
Text in subsection
START
-> Section A
QUIT
```
