# Whitespace Tolerance Around `=`

A `set` statement tolerates arbitrary whitespace (or none) on either side
of the `=` token.

## Script
```cuentitos
--- variables
int a = 0
int b = 0
int c = 0
int d = 0
---
set a=1
set b =2
set c= 3
set d   =   4
Hello
```

## Input
```input
n
n
n
n
n
?
s
```

## Result
```result
START
Hello
a: 1
b: 2
c: 3
d: 4
END
```
