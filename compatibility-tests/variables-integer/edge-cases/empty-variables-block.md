# Edge Case: Empty `--- variables` Block

An empty `--- variables` block should parse successfully and declare no variables.
`?` then has nothing to print, so the runtime should emit a warning.
Line 0 is used because `?` is a CLI input with no corresponding script line.

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
empty-variables-block.cuentitos:0: WARNING: No variables declared.
This is the story.
END
```
