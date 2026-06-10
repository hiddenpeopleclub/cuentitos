# Set in Script Without a Variables Block

A `set` statement in a script that does not declare a `--- variables ---`
block at all is a parse-time error: nothing has been declared, so the
target name cannot resolve. This mirrors the integer rule (see
`variables-integer/errors/set-without-variables-block.md`); the
`Undefined variable` path is type-agnostic, so this already passes today and
serves as a regression guard.

## Script
```cuentitos
set door_open = true
Hello
```

## Input
```input
s
```

## Result
```result
set-without-variables-block.cuentitos:1: ERROR: Undefined variable: 'door_open'.
```
