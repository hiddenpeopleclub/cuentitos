# Jump Backward to Section Defined Earlier

This test verifies that jumping to a section defined earlier in the file works correctly.
Note: This creates an infinite loop, so we use 'n' commands and 'q' to quit after several iterations.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B

# Section C
-> Section A
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
START
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
-> Section A
Text in A
-> Section B
Text in B
-> Section C
QUIT
```
