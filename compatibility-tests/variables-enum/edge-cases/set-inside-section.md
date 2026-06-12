# Set Inside a Section

A `set` on an enum variable is valid inside a section body. The `set`
executes silently as the runtime enters the section, and a later `?` reflects
the change.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
# intro: Intro
set mood = sad
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
mood: sad
END
```
