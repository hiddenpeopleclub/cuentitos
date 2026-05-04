# Edge Case: Default Precedence — `a OR b AND c` Behaves as `a OR (b AND c)`

`AND` binds tighter than `OR`. Without parentheses, `a OR b AND c` must
evaluate identically to `a OR (b AND c)`. With these inputs, both produce
**true** (because `a > 0` alone satisfies the OR), so both gated lines are
shown. If precedence were inverted to `(a OR b) AND c`, the first line
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
  req a > 0 OR b > 0 AND c > 0
Equivalent explicit grouping.
  req a > 0 OR (b > 0 AND c > 0)
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
