# Duplicate Section Names - Different Levels Allowed

This test verifies that sections with the same name are allowed at different levels.

## Script
```cuentitos
# Chapter One
This is chapter one
  ## Chapter One
  This is a sub-section with the same name
    ### Chapter One
    This is a sub-sub-section with the same name
```

## Input
```input
s
```

## Result
```result
START
-> Chapter One
This is chapter one
-> Chapter One\Chapter One
This is a sub-section with the same name
-> Chapter One\Chapter One\Chapter One
This is a sub-sub-section with the same name
END
```
!!! disabled
