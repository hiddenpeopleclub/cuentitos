# Enum Mixed With Other Variable Types

Enum declarations can be interleaved with other variable types in the same
`--- variables` block. Each keeps its own initialization rule: integers
default to 0, enums stay unset until assigned.

## Script
```cuentitos
--- variables
int score = 10
enum mood = happy, sad, angry
int lives
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
score: 10
mood: <unset>
lives: 0
This is the story.
END
```
