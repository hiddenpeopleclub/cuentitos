# Error: S Command At Option Prompt

User enters 's' at option prompt - should show help message.

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
s
2
s
```

## Result
```result
START
Choose one
  1. Option A
  2. Option B
> Use option numbers (1-2) to choose (plus q to quit)
Choose one
  1. Option A
  2. Option B
> Selected: Option B
Content B
END
```
