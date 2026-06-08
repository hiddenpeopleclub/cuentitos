# Debug `?` Immediately After Load

`?` issued as the first input should print the initial values of every declared
float variable, in declaration order. Variables without a default show `0.0`.

## Script
```cuentitos
--- variables
float health = 42.5
float score
float delta = -3.0
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
health: 42.5
score: 0.0
delta: -3.0
First line.
Second line.
END
```
