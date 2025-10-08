# Option Indentation Display

Verify options display with 2-space indent under parent text.

## Script
```cuentitos
Parent text here
  * First option
    Content A
  * Second option
    Content B
```

## Input
```input
1
s
```

## Result
```result
START
Parent text here
  1. First option
  2. Second option
> Selected: First option
Content A
END
```
