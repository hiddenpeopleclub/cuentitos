# Edge Case: Default Precedence — `a or b and c` Behaves as `a or (b and c)`

`and` binds tighter than `or`. Without parentheses, `a or b and c` must
evaluate identically to `a or (b and c)`. With these inputs, both produce
**true** (because `a > 0` alone satisfies the `or`), so both gated lines
are shown. If precedence were inverted to `(a or b) and c`, the first line
would not be shown — its appearance proves AND-tighter-than-OR is in
effect.

## Script
```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

Implicit precedence.
  req a > 0 or b > 0 and c > 0
Equivalent explicit grouping.
  req a > 0 or (b > 0 and c > 0)
```

## Input
```input
s
```

## Result
```result
START
Implicit precedence.
Equivalent explicit grouping.
END
```
