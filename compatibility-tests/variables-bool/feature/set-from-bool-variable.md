# Set: Bool Variable on the RHS

A `set <var> = <other_bool>` reads the current value of another declared bool
variable and assigns it to the target. The RHS reference is resolved at
runtime, so the target takes the source's current value.

## Script
```cuentitos
--- variables
bool source = true
bool target = false
---
set target = source
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
source: true
target: true
END
```
