# CLI Call During Call Stack

This test verifies that CLI call-and-return works when already in a called section.

## Script
```cuentitos
# Section A
Text in A
<-> Section B
Text after first call in A

# Section B
Text in B
More text in B

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
<-> Section C
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
Text in C
More text in B
```
