# Require: Parentheses Group Logical Sub-Expressions

Parentheses force grouping in a `req` expression. With these variable values,
`(a > 0 or b > 0) and c > 0` evaluates to **false** because `c > 0` is false,
so the `and` short-circuits and the parent is hidden. The companion test
`logical-default-precedence.md` uses the same values without parentheses to
prove the parens change the outcome.

## Script
```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

With parentheses.
  req (a > 0 or b > 0) and c > 0
```

## Input
```input
s
```

## Result
```result
START
END
```
