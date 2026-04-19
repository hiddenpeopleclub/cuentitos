# Call Another Section (Non-Recursive)

This test verifies that a section can call another section using <->.

## Script
```cuentitos
# Section A
Count 1
<-> Section B

# Section B
In B
```

## Input
```input
s
```

## Result
```result
START
-> Section A
Count 1
-> Section B
In B
-> Section B
In B
END
```
