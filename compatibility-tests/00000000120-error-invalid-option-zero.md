# Error: Invalid Option Number - Zero

User enters zero at option prompt.

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
0
1
s
```

## Result
```result
START
Choose one
  1. Option A
  2. Option B
> Invalid option: 0
Choose one
  1. Option A
  2. Option B
> Selected: Option A
Content A
END
```
