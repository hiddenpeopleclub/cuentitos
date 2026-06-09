# Bool Variable With `true` Literal Default

A bool variable declared with `= true` should initialize to `true`.

## Script
```cuentitos
--- variables
bool starts_true = true
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
starts_true: true
This is the story.
END
```
