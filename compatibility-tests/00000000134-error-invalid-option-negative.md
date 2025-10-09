# Error: Invalid Option Number - Negative

User enters negative number at option prompt.

## Script
```cuentitos
Choose one
  * Option A
    Content A
  * Option B
    Content B
```

## Input
```input
-1
1
s
```

## Result
```result
START
Choose one
  1. Option A
  2. Option B
> Invalid option: -1
Choose one
  1. Option A
  2. Option B
> Selected: Option A
Content A
END
```
