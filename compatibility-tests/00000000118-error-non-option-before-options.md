# Error: Non-Option Before Options

Sibling content before options at same level should produce validation error.

## Script
```cuentitos
Parent text
  Regular child text
  * Option A
    Content A
  * Option B
    Content B
```

## Input
```input
s
```

## Result
```result
Error: Options must have a parent
```
