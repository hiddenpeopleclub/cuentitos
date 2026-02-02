# Variables With Script Content

Variables block at the top followed by normal script content works correctly.

## Script
```cuentitos
--- variables
int count = 5
---
First line
  Second line
```

## Input
```input
n,?,s
```

## Result
```result
START
First line
count: 5
Second line
END
```
