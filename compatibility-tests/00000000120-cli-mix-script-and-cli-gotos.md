# Mix Script and CLI GoTos

This test verifies that script gotos and CLI gotos work together seamlessly.

## Script
```cuentitos
# Section A
Text in A
-> Section C

# Section B
Text in B

# Section C
Text in C
```

## Input
```input
n
n
n
n
n
-> Section B
s
```

## Result
```result
START
-> Section A
Text in A
-> Section C
Text in C
-> Section B
Text in B
-> Section C
Text in C
END
```
