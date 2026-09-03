# Secondary Locale Renders Translated Text

Switching to a secondary locale with `locale <code>` replaces every
translated text block with the matching entry from that locale's
`--- translation ... ---` block. See
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
farewell: "Adios."
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
Adios.
END
```
