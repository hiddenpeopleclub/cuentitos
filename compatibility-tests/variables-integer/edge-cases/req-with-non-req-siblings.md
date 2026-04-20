# Require Edge Case: `req` Alongside Non-`req` Siblings

When a `req` shares a parent with non-`req` siblings (text lines) and the
`req` fails, **all** of the parent's descendants are skipped — including
those non-`req` siblings.

## Script
```cuentitos
--- variables
int x = 0
---

Gated parent.
  req x > 0
  Non-req child one.
  Non-req child two.
Sibling block — not gated, always shown.
```

## Input
```input
s
```

## Result
```result
START
Sibling block — not gated, always shown.
END
```
