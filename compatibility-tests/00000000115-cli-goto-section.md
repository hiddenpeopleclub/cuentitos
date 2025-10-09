# CLI GoTo Section

This test verifies that a user can type a goto command in CLI to jump to a section.

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
-> Section B
s
```

## Result
```result
START
-> Section A
Text in A
-> Section B
Text in B
END
```
