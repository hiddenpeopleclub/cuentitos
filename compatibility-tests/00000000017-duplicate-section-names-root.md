# Duplicate Section Names Error - Root Level

This test verifies that sections at the root level cannot have the same name.

## Script
```cuentitos
# Introduction
This is the first introduction

# Chapter One
This is chapter one

# Introduction
This should cause an error
```

## Input
```input
n
```

## Result
```result
7: ERROR: Duplicate section name: 'Introduction' already exists at root level. Previously defined at line 1.
```
