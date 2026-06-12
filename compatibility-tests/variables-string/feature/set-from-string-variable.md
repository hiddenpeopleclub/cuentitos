# Set: String Variable on the RHS

A `set <var> = <other_string>` reads the current value of another declared
string variable and assigns it to the target. The RHS reference is a bare
identifier (no quotes) resolved at runtime, so the target takes the
source's current value.

## Script
```cuentitos
--- variables
string source = "Aria"
string target = "Hero"
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
source: "Aria"
target: "Aria"
END
```
