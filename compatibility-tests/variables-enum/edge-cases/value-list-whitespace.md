# Edge Case: Whitespace Variants In Value List

Whitespace around the `=`, around commas, and trailing on the line is
insignificant. Each value is trimmed, so `  happy ,sad,  angry  ` declares
exactly three values: `happy`, `sad`, and `angry`. If trimming were broken the
surrounding spaces would make a value an invalid identifier and the declaration
would error; instead it parses cleanly and the variable is unset until a `set`
runs. (Setting to a value declared with surrounding whitespace is exercised by
the `set` suite.)

Note: the trailing spaces after `angry` on the declaration line are
intentional and load-bearing — do not let an editor or formatter strip them.

## Script
```cuentitos
--- variables
enum mood =   happy ,sad,  angry  
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
mood: <unset>
This is the story.
END
```
