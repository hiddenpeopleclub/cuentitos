# CLI Error: Trailing Backslash

This test verifies that CLI shows error for goto with trailing backslash.

## Script
```cuentitos
# Section A
Text in A
```

## Input
```input
n
n
-> Section A \
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
