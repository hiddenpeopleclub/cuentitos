# Edge Case: Two Enums Sharing A Value Name

Enum values are scoped to their own variable, so two enums may both declare a
value with the same name (`happy` here) without colliding. The duplicate-value
check is per-enum, not global: declaring `happy` in both `mood` and `weather`
raises no error. Both stay unset until a `set` runs; assigning each (and
proving the references are unambiguous) is exercised by the `set` suite.

## Script
```cuentitos
--- variables
enum mood = happy, sad
enum weather = sunny, happy
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
