# Edge Case: Two Enums Sharing A Value Name

Enum values are scoped to their own variable, so two enums may both declare a
value with the same name (`happy` here). References are always qualified by
the variable, so assigning `happy` to each is unambiguous.

## Script
```cuentitos
--- variables
enum mood = happy, sad
enum weather = sunny, happy
---
set mood = happy
set weather = happy
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
mood: happy
weather: happy
END
```
