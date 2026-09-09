# Error: Named Bucket Name Is Not Snake Case

A named bucket's name is an identifier and must be snake case: lower case
words joined by underscores. This name is camel case, so the compiler
rejects it.

## Script
```cuentitos
I step outside to check the sky.
[AfternoonWeather]
  (50%) It rains all afternoon.
  (50%) The sun holds until dusk.
```

## Input
```input
s
```

## Result
```result
named-bucket-name-not-snake-case.cuentitos:2: ERROR: Invalid bucket: name 'AfternoonWeather' must be snake case.
```
