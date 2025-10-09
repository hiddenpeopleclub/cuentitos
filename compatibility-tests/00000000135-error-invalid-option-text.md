# Error: Invalid Option Input - Text

User enters non-numeric text at option prompt.

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
abc
1
s
```

## Result
```result
START
Choose one
  1. Option A
  2. Option B
> Invalid option: abc
Choose one
  1. Option A
  2. Option B
> Selected: Option A
Content A
END
```
