# Operator Precedence

An expression with multiple variables applies standard arithmetic precedence
(`*` and `/` bind tighter than `+` and `-`).

## Script
```cuentitos
--- variables
int a = 5
int b = 2
int c = 3
int result
---
set result = a + b * c
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
a: 5
b: 2
c: 3
result: 11
END
```
