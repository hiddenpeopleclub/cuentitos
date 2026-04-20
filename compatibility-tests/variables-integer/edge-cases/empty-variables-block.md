# Edge Case: Empty `--- variables` Block

An empty `--- variables` block should parse successfully and declare no variables.
`?` should print nothing because there are no variables to display.

## Script
```cuentitos
--- variables
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
This is the story.
END
```
