# Enum With A Single Value

An enum's value list must have at least one value, and exactly one is allowed —
no comma is required. `enum mood = happy` is the lower boundary of the value
list: a valid declaration with a single value. The variable is unset until a
`set` runs, so `?` reports it as `<unset>`.

## Script
```cuentitos
--- variables
enum mood = happy
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
