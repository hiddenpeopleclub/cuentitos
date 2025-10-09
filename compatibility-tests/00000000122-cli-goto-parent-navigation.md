# CLI GoTo with Parent Navigation

This test verifies that a user can use .. to navigate to parent section.

## Script
```cuentitos
# Parent
Text in parent
  ## Child
  Text in child
  More text in child
```

## Input
```input
n
n
n
n
n
-> ..
s
```

## Result
```result
START
-> Parent
Text in parent
-> Parent \ Child
Text in child
More text in child
-> Parent
Text in parent
-> Parent \ Child
Text in child
More text in child
END
```
