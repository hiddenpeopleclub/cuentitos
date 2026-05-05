# Require: Default Precedence — `and` Binds Tighter Than `or`

Without parentheses, `a or b and c` parses as `a or (b and c)` because `and`
binds tighter than `or`. With these variable values, that evaluates to
**true** even though `c > 0` is false — `a > 0` alone is enough to satisfy
the `or`. The companion test `logical-paren-grouping.md` uses the same
values with explicit parentheses around `(a or b)` and produces the
opposite outcome, proving the parens change the result.

## Script
```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

Without parentheses.
  req a > 0 or b > 0 and c > 0
```

## Input
```input
s
```

## Result
```result
START
Without parentheses.
END
```
