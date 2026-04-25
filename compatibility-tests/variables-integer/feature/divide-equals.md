# Set Divide Equals

`set <var> /= <expr>` divides a declared integer variable in place.
Division truncates toward zero.

## Script
```cuentitos
--- variables
int score = 20
---
set score /= 4
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
score: 5
END
```
