# Section Without Title Error

This test verifies that a section without a title produces an error.

## Script
```cuentitos
# Valid Section
This is text in a valid section

#
This should cause an error
```

## Input
```input
n
```

## Result
```result
3: ERROR: Section without title: found empty section title.
```
