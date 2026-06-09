# Error: Duplicate Enum Variable Name

Declaring the same enum variable name twice in a `--- variables` block should
fail, mirroring the rule for every other variable type. The duplicate
declaration short-circuits the rest of the script.

## Script
```cuentitos
--- variables
enum mood = happy, sad
enum weather = sunny, rainy
enum mood = calm, tense
---

This is the story.
```

## Input
```input
s
```

## Result
```result
duplicate-variable-name.cuentitos:4: ERROR: Duplicate variable name: 'mood' already declared. Previously declared at line 2.
```
