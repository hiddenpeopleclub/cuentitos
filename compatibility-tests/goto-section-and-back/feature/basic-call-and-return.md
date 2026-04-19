# Basic Call and Return

This test verifies that a basic call and return works - jump to section, execute it, return to caller.

## Script
```cuentitos
# Section A
Text in A
<-> Section B
Text after call in A
-> END

# Section B
Text in B
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
Text after call in A
END
```
