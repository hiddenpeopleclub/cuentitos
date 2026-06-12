# Edge Case: Set From a Variable Copies By Value

`set <target> = <source>` copies the source's *current* value into the
target; it does not create an alias. Mutating the source afterward leaves
the previously-assigned target unchanged.

## Script
```cuentitos
--- variables
string source = "Aria"
string echo = "init"
---
set echo = source
set source = "Brenn"
Hello
```

## Input
```input
n
?
s
```

## Result
```result
START
Hello
source: "Brenn"
echo: "Aria"
END
```
