# CLI Error: No Space After Arrow

This test verifies that CLI shows error for goto without space after arrow.

## Script
```cuentitos
# Section A
Text in A
```

## Input
```input
n
n
->Section A
n
s
```

## Result
```result
ERROR: Invalid goto command: Expected section name after '->'
Cannot skip - reached the end of the script.
START
-> Section A
Text in A
END
```
