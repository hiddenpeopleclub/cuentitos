# Section Indentation Jump Error

A section that jumps indentation levels (4 spaces without a parent at 2 spaces) should fail.

## Script
```cuentitos
# Section A
    ## Too Deep
```

## Input
```input
n
```

## Result
```result
00000000008-section-indentation-jump-error.cuentitos:2: ERROR: Invalid indentation: found 4 spaces in: ## Too Deep
```
