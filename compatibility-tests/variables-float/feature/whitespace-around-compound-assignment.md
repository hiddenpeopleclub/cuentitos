# Whitespace Tolerance Around Compound Assignment

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) tolerate
arbitrary whitespace (or none) on either side, just like `=`.

## Script
```cuentitos
--- variables
float a = 10.0
float b = 10.0
float c = 10.0
float d = 10.0
---
set a+=1.0
set b -=2.0
set c*= 3.0
set d   /=   2.0
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
a: 11.0
b: 8.0
c: 30.0
d: 5.0
END
```
