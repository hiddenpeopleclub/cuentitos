# Call Section Then Jump to END

This test verifies that a section can call another section and then jump to END.

## Script
```cuentitos
# Section A
Text in A
<-> Section B
After call
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
After call
END
```
