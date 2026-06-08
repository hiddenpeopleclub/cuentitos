# String Variable With Literal Default

A string variable declared with a double-quoted literal default should
initialize to that literal. `?` renders strings double-quoted.

## Script
```cuentitos
--- variables
string name = "Aria"
---

This is the story.
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
This is the story.
END
```
