# Duplicate Section Names Error - Root Level

This test verifies that sections at the root level cannot have the same name.

## Script
```cuentitos
# Chapter One
This is chapter one
# Chapter Two
This is chapter two
# Chapter One
This should cause an error
```

## Input
```input
s
```

## Result
```result
test.cuentitos:5: ERROR: Duplicate section name: 'Chapter One' already exists at this level under '<root>'. Previously defined at line 1.
```
!!! disabled
