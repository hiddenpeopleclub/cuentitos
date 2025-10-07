# Error: Call Section Not Found

This test verifies that calling a non-existent section produces an error.

## Script
```cuentitos
# Section A
In A
<-> NonExistent
```

## Input
```input
s
```

## Result
```result
test.cuentitos:3: ERROR: Section not found: NonExistent
```
