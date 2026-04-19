# Error: Invalid Option Number - Too High

User enters number beyond available options range.

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
3
1
s
```

## Result
```result
START
Choose one
  1. Option A
  2. Option B
> Invalid option: 3
Choose one
  1. Option A
  2. Option B
> Selected: Option A
Content A
END
```
