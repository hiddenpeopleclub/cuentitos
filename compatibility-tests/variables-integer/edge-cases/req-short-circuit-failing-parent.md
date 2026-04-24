# Require Edge Case: Short-Circuit Under a Failing Parent `req`

When a parent block's `req` fails, none of the child block's `req` expressions
are evaluated — including expressions that would otherwise produce a runtime
error. This pins short-circuit semantics: the engine must skip the entire
descendant subtree without evaluating it.

If the engine evaluated descendants eagerly, the inner `10 / inner_zero` would
produce a runtime division-by-zero error and the script would abort before
`After.`. The expected output shows the script terminates cleanly, proving the
inner `req` was never reached.

## Script
```cuentitos
--- variables
int outer = 0
int inner_zero = 0
---

Outer fails — descendants must not be evaluated.
  req outer = 1
  Never shown.
    req inner_zero = 10 / inner_zero
After.
```

## Input
```input
s
```

## Result
```result
START
After.
END
```
