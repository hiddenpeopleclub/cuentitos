# Duplicate Section Names Error - Multiple Errors

This test verifies that multiple duplicate section name errors are reported correctly.

## Script
```cuentitos
# Story
  ## Chapter One
  This is chapter one
  ## Chapter Two
  This is chapter two
  ## Chapter One
  This should cause an error
# Story
This should also cause an error
```

## Input
```input
s
```

## Result
```result
test.cuentitos:6: ERROR: Duplicate section name: 'Chapter One' already exists at this level under 'Story'. Previously defined at line 2.

test.cuentitos:8: ERROR: Duplicate section name: 'Story' already exists at this level under '<root>'. Previously defined at line 1.
```
