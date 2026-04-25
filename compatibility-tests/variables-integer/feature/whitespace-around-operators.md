# Whitespace Tolerance Within Expressions

Whitespace around arithmetic operators inside a `set` expression is
optional and may be inconsistent within the same expression.

## Script
```cuentitos
--- variables
int a = 0
int b = 0
int c = 0
---
set a = 1+2*3
set b =  4 *5+ 6
set c = (1+ 2) *( 3 +4 )
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
a: 7
b: 26
c: 21
END
```
