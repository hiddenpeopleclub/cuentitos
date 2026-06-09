# Default Referencing an Earlier String Variable

A string default may reference a variable declared earlier in the same
block. The reference is a bare identifier (no quotes) and copies the
referenced value.

## Script
```cuentitos
--- variables
string hero_name = "Aria"
string echo = hero_name
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
hero_name: "Aria"
echo: "Aria"
This is the story.
END
```
