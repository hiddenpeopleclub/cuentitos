# Require: `set` Earlier in the Script Flips a Later `req`

A `set` executed before a `req` changes whether that `req` passes. The
comparison is evaluated using the enum variable's current runtime value, so
re-assigning the variant flips which guarded lines render. An enum has no
default, so the variable is assigned up front before the first comparison.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = happy
First check.
  req mood = happy
Second check.
  req mood = sad
set mood = sad
Third check.
  req mood = happy
Fourth check.
  req mood = sad
```

## Input
```input
s
```

## Result
```result
START
First check.
Fourth check.
END
```
