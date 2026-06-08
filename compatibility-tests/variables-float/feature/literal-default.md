# Float Variable With Literal Default

A float variable declared with a literal default should initialize to that
literal. Float literals use the `<digits>.<digits>` form.

## Script
```cuentitos
--- variables
float starting_health = 10.5
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
starting_health: 10.5
This is the story.
END
```
