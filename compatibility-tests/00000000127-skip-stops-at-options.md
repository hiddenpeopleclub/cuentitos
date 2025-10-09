# Skip Stops At Options

Skip command stops when encountering options.

## Script
```cuentitos
First line
Second line
Choose something
  * Option A
    Result A
  * Option B
    Result B
```

## Input
```input
s
1
s
```

## Result
```result
START
First line
Second line
Choose something
  1. Option A
  2. Option B
> Selected: Option A
Result A
END
```
