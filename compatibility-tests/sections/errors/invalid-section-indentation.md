# Invalid Section Indentation Error

This test verifies that sections with invalid indentation (not multiples of 2 spaces) produce an error.

## Script
```cuentitos
# Main Section
This is text in the main section
   ## Invalid Sub-section
   This should cause an error
```

## Input
```input
s
```

## Result
```result
invalid-section-indentation.cuentitos:3: ERROR: Invalid indentation: found 3 spaces.

invalid-section-indentation.cuentitos:4: ERROR: Invalid indentation: found 3 spaces.
```
