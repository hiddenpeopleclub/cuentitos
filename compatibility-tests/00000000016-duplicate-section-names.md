# Duplicate Section Names Error - Same Level Same Parent

This test verifies that sections at the same level under the same parent cannot have the same name.

## Script
```cuentitos
# Story
  ## Chapter One
  This is chapter one
  ## Chapter Two
  This is chapter two
  ## Chapter One
  This should cause an error
```

## Input
```input
s
```

## Result
```result
test.cuentitos:6: ERROR: Duplicate section name: 'Chapter One' already exists at this level under 'Story'. Previously defined at line 2.
```
!!! disabled
