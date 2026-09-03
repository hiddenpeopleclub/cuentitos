# Missing Translation Surfaces a Marker

When the active locale's `--- translation ... ---` block has no entry for a
key, that one block renders as `MISSING TRANSLATION <key>` instead of any
text. Other blocks in the same run that do have a translation still render
normally. See
`compatibility-tests/i18n-for-strings/feature/render-in-default-locale.md`
for the full grammar this test relies on.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

--- translation es
greeting: "Hola mundo."
---

Hello world. [greeting]
Goodbye. [farewell]
```

## Input
```input
locale es
s
```

## Result
```result
START
Hola mundo.
MISSING TRANSLATION farewell
END
```
