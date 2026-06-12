# Edge Case: Variant Resolution Is Scoped to the Target Enum

Two enums may declare a value with the same name (`happy` here). On a `set`,
the bare variant identifier is resolved against the *target* enum's value
list, not globally. `set weather = happy` therefore assigns weather's own
`happy`, unambiguously, even though `mood` also declares `happy`. This is the
`set`-side counterpart to the declaration edge case in
`edge-cases/enums-share-value-name.md`.

## Script
```cuentitos
--- variables
enum mood = happy, sad
enum weather = sunny, happy
---
set mood = happy
set weather = happy
This is the story.
```

## Input
```input
?
n
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
