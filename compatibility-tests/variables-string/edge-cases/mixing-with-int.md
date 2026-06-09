# Edge Case: Mixing String With Int

A `--- variables` block may declare string variables alongside int
variables. Declaration order is preserved for `?`, and each type renders
in its own format: integers bare, strings double-quoted.

Unlike a cross-type test that also spans bool/float, this case depends
only on `int` (already implemented) and `string`, so it can go green on
the string implementation alone — making it a clean TDD signal for the
string task.

## Script
```cuentitos
--- variables
int count = 7
string name = "Aria"
int score
string title = "Hero"
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
count: 7
name: "Aria"
score: 0
title: "Hero"
This is the story.
END
```
