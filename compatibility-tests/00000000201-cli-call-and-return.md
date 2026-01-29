# CLI Call and Return

This test verifies that a user can type a call-and-return command in CLI.

## Script
```cuentitos
# Section A
Text in A
More text in A

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
<-> Section B
s
```

## Result
```result
START
-> Section A
Text in A
More text in A
-> Section B
Text in B
-> Section B
```
