# Jump from Indented Context with Relative Path

This test verifies that relative path resolution works correctly based on indentation context.

## Script
```cuentitos
# Root A
  ## Section X
  Text in X
  -> Section Y
  ## Section Y
  Text in Y

# Root B
  ## Section Y
  Text in different Y
```

## Input
```input
s
```

## Result
```result
START
-> Root A
-> Root A \ Section X
Text in X
-> Root A \ Section Y
Text in Y
-> Root B
-> Root B \ Section Y
Text in different Y
END
```
