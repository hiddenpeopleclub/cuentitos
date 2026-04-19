# Multiple Calls in Sequence

This test verifies that multiple call-and-return commands work in sequence.

## Script
```cuentitos
# Section A
Start A
<-> Section B
Middle A
<-> Section C
End A
-> END

# Section B
In B

# Section C
In C
```

## Input
```input
s
```

## Result
```result
START
-> Section A
Start A
-> Section B
In B
Middle A
-> Section C
In C
End A
END
```
