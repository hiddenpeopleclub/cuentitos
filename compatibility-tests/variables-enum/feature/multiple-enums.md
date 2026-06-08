# Multiple Enums In One Block

A `--- variables` block may declare several enum variables; declaration order
is preserved for `?`. Each is unset until a `set` runs.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
enum weather = sunny, rainy
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
weather: <unset>
This is the story.
END
```
