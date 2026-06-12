# Set Error: Value Not a Declared Variant

A `set` on an enum variable accepts only a variant that appears in the enum's
declared value list. A bare identifier that is not one of those variants is a
parse-time error — the value is rejected before the runtime ever runs, so no
`START`/`END` is printed.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = ecstatic
This is the story.
```

## Input
```input
s
```

## Result
```result
set-invalid-variant.cuentitos:4: ERROR: Invalid value: 'ecstatic' is not a variant of enum mood.
```
