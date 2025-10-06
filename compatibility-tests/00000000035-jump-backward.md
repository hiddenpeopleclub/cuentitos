# Jump Backward to Section Defined Earlier

This test verifies that jumping to a section defined earlier in the file works correctly.

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
s
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
-> Section A
Text in A
-> Section B
Text in B
-> Section C
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
