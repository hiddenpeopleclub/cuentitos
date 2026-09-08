# Error: Named Bucket Has No Branches

A named bucket holds all of its content in its children. A group whose only
children are modifiers has nothing to draw from, so the compiler rejects it.

## Script
```cuentitos
--- variables
bool market_open = true
---
I step outside to check the sky.
[afternoon_weather]
  req market_open
```

## Input
```input
s
```

## Result
```result
named-bucket-with-no-branches.cuentitos:5: ERROR: Invalid bucket: named bucket 'afternoon_weather' has no branches.
```
