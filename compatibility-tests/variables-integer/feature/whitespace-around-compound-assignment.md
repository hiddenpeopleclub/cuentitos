# Whitespace Tolerance Around Compound Assignment

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) tolerate
arbitrary whitespace (or none) on either side, just like `=`.

## Script
```cuentitos
--- variables
int a = 10
int b = 10
int c = 10
int d = 10
---
set a+=1
set b -=2
set c*= 3
set d   /=   2
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
a: 11
b: 8
c: 30
d: 5
END
```
