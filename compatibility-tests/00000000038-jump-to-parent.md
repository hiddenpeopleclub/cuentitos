# Jump to Parent Using ..

This test verifies that jumping to a parent section using .. works correctly.

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
s
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
