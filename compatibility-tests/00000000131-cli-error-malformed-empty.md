# CLI Error: Malformed GoTo Empty

This test verifies that CLI shows error for goto with no section name.

## Script
```cuentitos
# Section A
Text in A
```

## Input
```input
n
n
->
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
