# Bool Variable With `false` Literal Default

A bool variable declared with `= false` should initialize to `false`.

## Script
```cuentitos
--- variables
bool starts_false = false
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
starts_false: false
This is the story.
END
```
