# Single Enum Declaration

A single enum variable can be declared with its allowed values using
`enum <name> = <value1>, <value2>, ...`. Until a `set` runs the variable is
unset, so `?` reports it as `<unset>`.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
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
mood: <unset>
This is the story.
END
```
