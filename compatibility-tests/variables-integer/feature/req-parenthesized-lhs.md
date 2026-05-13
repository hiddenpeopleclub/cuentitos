# Require: Parenthesized Arithmetic on the LHS

A `req` may use a parenthesized arithmetic expression on the left-hand
side, including continuations like `(a + b) + 1`, `(a + b) * 2`, and
`(a) + 1`. The leading `(` could in principle open either a boolean
group (`(x > 0) and y > 0`) or the LHS of a comparison; the boolean
parser disambiguates by scanning past the matched `)` through any
arithmetic continuation tokens and checking whether a comparison
operator follows.

## Script
```cuentitos
--- variables
int a = 2
int b = 3
int c = 5
---

Sum-of-grouped-and-then-extra is greater than c.
  req (a + b) + 1 > c
Product-of-grouped-times-two equals ten.
  req (a + b) * 2 = 10
Single-var paren plus one is greater than zero.
  req (a) + 1 > 0
Nested-parens plus one is greater than zero.
  req ((a + b)) + 1 > 0
Sum-of-grouped-and-then-extra does not beat thirty.
  req (a + b) + 1 > 30
```

## Input
```input
s
```

## Result
```result
START
Sum-of-grouped-and-then-extra is greater than c.
Product-of-grouped-times-two equals ten.
Single-var paren plus one is greater than zero.
Nested-parens plus one is greater than zero.
END
```
