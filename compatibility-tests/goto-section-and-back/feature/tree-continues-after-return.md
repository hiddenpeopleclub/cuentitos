# Normal Tree Continues After Return

This test verifies that normal tree traversal continues after a call returns.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A
  ## Child of A
  In Child of A

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
In A
-> Section B
In B
Back in A
-> Section A \ Child of A
In Child of A
-> Section B
In B
END
```
