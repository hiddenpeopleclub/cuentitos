# Jump to Section with Subsections Executes All

This test verifies that jumping to a section with subsections executes all of them.

## Script
```cuentitos
# Section A
-> Section B

# Section B
Text in B
  ## Sub B1
  Text in sub B1
  ## Sub B2
  Text in sub B2

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
-> Section B
Text in B
-> Section B \ Sub B1
Text in sub B1
-> Section B \ Sub B2
Text in sub B2
-> Section C
Text in C
END
```
