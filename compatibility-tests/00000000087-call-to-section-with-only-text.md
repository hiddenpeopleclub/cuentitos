# Call to Section with Only Text

This test verifies that calling a section with only text (no children) works correctly.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
Only text in B
```

## Input
```input
s
```

## Result
```result
START
-> Section A
In A
-> Section B
Only text in B
Back in A
-> Section B
Only text in B
END
```
