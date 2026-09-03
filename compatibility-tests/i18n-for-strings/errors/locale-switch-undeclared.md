# Error: Switching to an Undeclared Locale

Passing `locale <code>` where `<code>` never appears in the `locales:` list
is a runtime input mistake rather than a script defect, so it produces a
`WARNING:` at the point the command runs, not a parse-time `ERROR:`. The
active locale is unchanged, so the story still renders in the default
locale.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

Hello world. [greeting]
```

## Input
```input
locale fr
s
```

## Result
```result
locale-switch-undeclared.cuentitos:0: WARNING: Invalid locale "fr": expected one of the declared locales.
START
Hello world.
END
```
