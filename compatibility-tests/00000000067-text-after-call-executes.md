# Text After Call Executes

This test verifies that text after a call-and-return command is reachable (unlike regular goto).

## Script
```cuentitos
# Section A
Before call
<-> Section B
After call
More after call
-> END

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
Before call
-> Section B
In B
After call
More after call
END
```
