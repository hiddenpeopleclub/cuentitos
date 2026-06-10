# Set Inside a Section

A `set` statement is valid inside a section body. The `set` executes silently
as the runtime enters the section, and a later `?` reflects the change.

## Script
```cuentitos
--- variables
bool flag = false
---
# intro: Intro
set flag = true
The intro.
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
-> Intro
The intro.
flag: true
END
```
