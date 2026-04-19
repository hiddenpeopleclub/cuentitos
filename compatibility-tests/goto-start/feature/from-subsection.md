# Jump to START from Within Subsection

This test verifies that -> START works from within nested sections.

## Script
```cuentitos
# Section A
Text in A
  ## Subsection
  Text in subsection
  -> START

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
