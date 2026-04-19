# CLI GoTo Current Section

This test verifies that a user can use . to re-enter current section.

## Script
```cuentitos
# Section A
Text in A
More text in A
```

## Input
```input
n
n
n
-> .
s
```

## Result
```result
START
-> Section A
Text in A
More text in A
-> Section A
Text in A
More text in A
END
```
