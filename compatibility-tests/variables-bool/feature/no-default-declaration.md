# Bool Variable Declaration Without Default

A bool variable declared without a default value should initialize to `false`.

## Script
```cuentitos
--- variables
bool a_default_bool
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
a_default_bool: false
This is the story.
END
```
