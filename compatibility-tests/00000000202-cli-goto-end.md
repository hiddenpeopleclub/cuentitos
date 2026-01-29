# CLI GoTo END

This test verifies that a user can type goto END to jump to the end.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B
```

## Input
```input
n
n
-> END
```

## Result
```result
START
-> Section A
Text in A
END
```
