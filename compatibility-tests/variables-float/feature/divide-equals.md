# Set Divide Equals

`set <var> /= <expr>` divides a declared float variable in place. Unlike
integer division, float division does **not** truncate, so `7.0 / 2.0`
is `3.5`.

## Script
```cuentitos
--- variables
float score = 7.0
---
set score /= 2.0
Hello
```

## Input
```input
n
?
s
```

## Result
```result
START
Hello
score: 3.5
END
```
