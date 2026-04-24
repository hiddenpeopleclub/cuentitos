# Division Truncation Toward Zero

Integer division truncates toward zero, including when operands are negative.
This distinguishes from floor division: `-7 / 2` is `-3`, not `-4`.

## Script
```cuentitos
--- variables
int pos_pos
int neg_pos
int pos_neg
int neg_neg
---
set pos_pos = 7 / 2
set neg_pos = -7 / 2
set pos_neg = 7 / -2
set neg_neg = -7 / -2
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
pos_pos: 3
neg_pos: -3
pos_neg: -3
neg_neg: 3
END
```
