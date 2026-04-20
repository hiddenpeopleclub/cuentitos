# Error: Malformed `--- variables` Block Delimiters

A `--- variables` block that is opened but never closed with `---` should fail.

## Script
```cuentitos
--- variables
int a = 1

This is the story.
```

## Input
```input
s
```

## Result
```result
malformed-delimiters.cuentitos:1: ERROR: Unterminated '--- variables' block: missing closing '---'.
```
