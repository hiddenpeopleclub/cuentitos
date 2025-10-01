# Sub-section Without Parent Error

This test verifies that a sub-section without a parent section produces an error.

## Script
```cuentitos
  ## Orphaned Sub-section
  This should cause an error

# Valid Section
This is text in a valid section
  ## Valid Sub-section
  This is valid
```

## Input
```input
s
```

## Result
```result
test.cuentitos:1: ERROR: Invalid section hierarchy: found sub-section without parent section.
```
