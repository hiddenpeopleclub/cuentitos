# Debug `?` After Navigating Through Some Blocks

`?` issued mid-execution should still print the initial values (no `set` exists yet, so defaults are retained).

## Script
```cuentitos
--- variables
int count = 42
int score = 100
---

First line.
Second line.
Third line.
```

## Input
```input
n
n
?
s
```

## Result
```result
START
First line.
Second line.
count: 42
score: 100
Third line.
END
```
