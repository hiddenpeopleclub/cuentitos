# Require: Default Precedence — AND Binds Tighter Than OR

Without parentheses, `a OR b AND c` parses as `a OR (b AND c)` because `AND`
binds tighter than `OR`. With these variable values, that evaluates to
**true** even though `c > 0` is false — `a > 0` alone is enough to satisfy
the OR. The companion test `logical-paren-grouping.md` uses the same values
with explicit parentheses around `(a OR b)` and produces the opposite
outcome, proving the parens change the result.

## Script
```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

Without parentheses.
  req a > 0 OR b > 0 AND c > 0
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
