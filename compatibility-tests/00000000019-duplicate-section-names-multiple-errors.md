# Duplicate Section Names Error - Multiple Errors

This test verifies that multiple duplicate section name errors are reported correctly.

## Script
```cuentitos
# Introduction
This is the first introduction

# Story
  ## Chapter One
  This is chapter one
  ## Chapter One
  First duplicate error
  ## Chapter One
  Second duplicate error

# Introduction
Second root level duplicate

# Story
Third root level duplicate
```

## Input
```input
n
```

## Result
```result
6: ERROR: Duplicate section name: 'Chapter One' already exists at this level under 'Story'. Previously defined at line 5.
8: ERROR: Duplicate section name: 'Chapter One' already exists at this level under 'Story'. Previously defined at line 5.
10: ERROR: Duplicate section name: 'Introduction' already exists at root level. Previously defined at line 1.
12: ERROR: Duplicate section name: 'Story' already exists at root level. Previously defined at line 4.
```
