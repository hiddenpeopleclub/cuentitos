# Query Enum After Set

The `?` command reflects an enum's value after a `set` assigns one of its
declared values. The bare value on the right of `=` is resolved against the
variable's own value list.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
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
