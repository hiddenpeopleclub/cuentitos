# Invalid Indentation Error

A script with invalid indentation (3 spaces instead of 2) should result in a parse error.

## Script
```cuentitos
First line
   Invalid indentation
```

## Input
```input
n
```

## Result
```result
00000000010-invalid-indentation-error.cuentitos:2: ERROR: Invalid indentation: found 3 spaces.
```
