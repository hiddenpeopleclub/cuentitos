# Debug `?` After Navigating Through Some Blocks

`?` issued mid-execution should still print the initial values (no `set`
on strings exists yet, so defaults are retained), double-quoted and in
declaration order.

## Script
```cuentitos
--- variables
string name = "Aria"
string title = "Hero"
---

First line.
Second line.
Third line.
```

## Input
```input
n
n
?
s
```

## Result
```result
START
First line.
Second line.
name: "Aria"
title: "Hero"
Third line.
END
```
