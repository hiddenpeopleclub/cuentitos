# Call to Section That Only Contains Call

This test verifies that calling a section that only contains another call works correctly.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
<-> Section C

# Section C
In C
```

## Input
```input
s
```

## Result
```result
test.cuentitos:6: ERROR: Section must contain at least one block: Section B
```
