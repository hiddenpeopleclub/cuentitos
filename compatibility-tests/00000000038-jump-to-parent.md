# Jump to Parent Using ..

This test verifies that jumping to a parent section using .. works correctly.
Note: This creates an infinite loop, so we use 'n' commands and 'q' to quit after several iterations.

## Script
```cuentitos
# Parent
Text in parent
  ## Child
  Text in child
  -> ..
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
q
```

## Result
```result
START
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
-> Parent
QUIT
```
