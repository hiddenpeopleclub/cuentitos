# Locale Declared With No Translation Block

A locale that appears in the `locales:` list needs no `--- translation
... ---` block of its own; a declared locale with no translation block at
all has zero translations. Every keyed text block then renders `MISSING
TRANSLATION <key>` while that locale is active.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

Hello. [greeting]
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
MISSING TRANSLATION greeting
MISSING TRANSLATION farewell
END
```
