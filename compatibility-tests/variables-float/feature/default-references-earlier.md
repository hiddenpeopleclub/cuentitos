# Default Referencing an Earlier Variable

A float default expression may reference a float variable declared earlier in
the same block. This mirrors the feature summary's
`float bonus = starting_health * 2.0`.

## Script
```cuentitos
--- variables
float starting_health = 10.5
float bonus = starting_health * 2.0
float copy = starting_health
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
starting_health: 10.5
bonus: 21.0
copy: 10.5
This is the story.
END
```
