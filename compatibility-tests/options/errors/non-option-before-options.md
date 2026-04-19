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
non-option-before-options.cuentitos:3: ERROR: Options must have a parent

non-option-before-options.cuentitos:5: ERROR: Options must have a parent
```
