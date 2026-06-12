# Query After Set

The `?` CLI command reflects the updated value of an enum variable after a
`set`, rendered as the bare variant name.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = sad
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
mood: sad
END
```
