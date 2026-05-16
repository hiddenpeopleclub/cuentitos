# Error: Integer Literal Overflow in a Default

A default expression whose single offending literal exceeds the integer
range is a parse-time error. Previously this folded into the generic
"Integer overflow in default expression" diagnostic; the message now
names the offending literal — parallel to the matching `set` and `req`
diagnostics — so the author knows the literal, not the surrounding
syntax, is the problem.

## Script
```cuentitos
--- variables
int x = 99999999999999999999
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-literal-overflow.cuentitos:2: ERROR: Integer overflow in default expression for 'x': literal '99999999999999999999' exceeds the integer range.
```
