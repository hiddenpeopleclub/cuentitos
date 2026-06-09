# String Variable Declaration Without Default

A string variable declared without a default value should initialize to
the empty string. `?` renders the empty string as `""`.

## Script
```cuentitos
--- variables
string name
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
name: ""
This is the story.
END
```
