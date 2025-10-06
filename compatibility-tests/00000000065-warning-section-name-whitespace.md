# Warning: Section Name with Leading/Trailing Whitespace

This test verifies that section names with leading/trailing whitespace generate a warning.
Note: This creates an infinite loop (section jumps to itself), so we use 'n' commands and 'q' to quit.

## Script
```cuentitos
#  Section A
Text in A
->  Section A

# Section B
Text in B
```

## Input
```input
n
n
n
n
n
n
n
n
n
n
q
```

## Result
```result
test.cuentitos:1: WARNING: Section name has leading/trailing whitespace: ' Section A'. Trimmed to 'Section A'
test.cuentitos:3: WARNING: Section name has leading/trailing whitespace: ' Section A'. Trimmed to 'Section A'
START
-> Section A
Text in A
-> Section A
Text in A
-> Section A
Text in A
-> Section A
QUIT
```
