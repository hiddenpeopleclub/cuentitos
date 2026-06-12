# Edge Case: Set to the Empty String

A `set <var> = ""` assigns the empty string, overwriting a non-empty
default. `?` renders the empty string as `""`.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = ""
Hello
```

## Input
```input
n
?
s
```

## Result
```result
START
Hello
name: ""
END
```
