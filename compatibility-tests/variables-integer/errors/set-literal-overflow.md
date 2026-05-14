# Set Error: Integer Literal Overflow

A `set` whose RHS contains an integer literal that exceeds the integer
range is a parse-time error. Previously this folded into the generic
"Malformed 'set' statement" diagnostic; now the message names the
offending literal — parallel to the matching `req` diagnostic — so the
author knows the literal, not the surrounding syntax, is the problem.

## Script
```cuentitos
--- variables
int x = 0
---

set x = 99999999999999999999
```

## Input
```input
s
```

## Result
```result
set-literal-overflow.cuentitos:5: ERROR: Integer overflow in 'set' expression: literal '99999999999999999999' exceeds the integer range.
```
