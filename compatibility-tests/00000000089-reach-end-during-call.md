# Reach END During Call

This test verifies that if END is reached during a call, the script terminates without returning.

## Script
```cuentitos
# Section A
In A
<-> Section B
This should not execute

# Section B
In B
-> END
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
In B
END
```
