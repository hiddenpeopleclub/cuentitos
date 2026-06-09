# Edge Case: Whitespace Variants In Value List

Messy whitespace around the `=`, around commas, and trailing on the line still
parses without error: `  happy ,sad,  angry  ` is a valid declaration and the
variable is unset, so `?` reports `mood: <unset>`.

This test only asserts that the declaration parses cleanly — declaration-scope
output (`<unset>`) cannot observe what each value was trimmed *to*, so it does
not prove the trimmed values are exactly `happy`, `sad`, `angry`. That proof
(setting to a value declared with surrounding whitespace and reading it back)
belongs to the `set` suite.

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
