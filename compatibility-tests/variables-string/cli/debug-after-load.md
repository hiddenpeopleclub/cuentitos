# Debug `?` Immediately After Load

`?` issued as the first input should print the initial values of every
declared string variable, in declaration order, double-quoted. A variable
declared without a default prints as the empty string `""`.

## Script
```cuentitos
--- variables
string name = "Aria"
string title
string greeting = "Hello"
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
name: "Aria"
title: ""
greeting: "Hello"
First line.
Second line.
END
```
