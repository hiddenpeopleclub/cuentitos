# Require Error: Arithmetic Expression Nests Too Deeply

The depth cap on `req` boolean expressions also covers the arithmetic
sublanguage on either side of a comparison — a long chain of unary
minuses or nested parens would otherwise grow the parser stack one
frame per token. Both layers share a single `MAX_EXPRESSION_DEPTH`
budget, so adversarial input can't dodge the cap by switching layers.

The chain below has 65 nested `(`s in the RHS — one past the 64-level
cap — so the parser bails out before recursion gets dangerous.

## Script
```cuentitos
--- variables
int x = 1
---

Line.
  req x > (((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((1)))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
```

## Input
```input
s
```

## Result
```result
req-arith-too-deep.cuentitos:6: ERROR: 'req' expression nests too deeply (max 64 levels).
```
