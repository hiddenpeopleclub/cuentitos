# Recursive Call with Dot

This test verifies that a section can call itself recursively using <-> .

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
