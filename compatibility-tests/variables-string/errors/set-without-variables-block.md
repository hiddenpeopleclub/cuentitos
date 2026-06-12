# Set in Script Without a Variables Block

A `set` statement in a script that does not declare a `--- variables ---`
block at all is a parse-time error: nothing has been declared, so the
target name cannot resolve.

## Script
```cuentitos
set name = "Aria"
Hello
```

## Input
```input
s
```

## Result
```result
set-without-variables-block.cuentitos:1: ERROR: Undefined variable: 'name'.
```
