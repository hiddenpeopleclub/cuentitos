# Call to Section with Subsections

This test verifies that calling a section executes all its subsections before returning.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
In B
  ## Subsection B1
  In B1
  ## Subsection B2
  In B2
```

## Input
```input
s
```

## Result
```result
START
-> Section A
In A
-> Section B
In B
-> Section B \ Subsection B1
In B1
-> Section B \ Subsection B2
In B2
Back in A
-> Section B
In B
-> Section B \ Subsection B1
In B1
-> Section B \ Subsection B2
In B2
END
```
