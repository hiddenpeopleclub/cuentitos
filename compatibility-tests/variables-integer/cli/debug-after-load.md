# Debug `?` Immediately After Load

`?` issued as the first input should print the initial values of every declared variable.

## Script
```cuentitos
--- variables
int count = 42
int score
int delta = -3
---

First line.
Second line.
```

## Input
```input
?
s
```

## Result
```result
START
count: 42
score: 0
delta: -3
First line.
Second line.
END
```
