# Translated Section Display Names Leave Jumps Intact

Section display names are translatable. The `-> The Docks` jump on line
8 resolves against the section identifier, which is always held in the Default
Locale, so the jump lands while the rendered heading appears in `es`. This is
the reason the parser has to store a section's implicit identifier as its own
database string rather than sharing the display name's `StringId`.

## ADRs

- [I18n via Compile-Time Translation Tables](../../../docs/architecture/000017-i18n-translation-tables.md)

## Pending

Blocked on the Compiler milestone. See ADR 000017.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

# The Market
You arrive at the market.
-> The Docks

# The Docks
Water everywhere.
```

## Translations
```es
id,line,original,translation,status
a798d7a3,6,The Market,El Mercado,
fa2156dc,7,You arrive at the market.,Llegas al mercado.,
4b61a71c,10,The Docks,Los Muelles,
8cad5e19,11,Water everywhere.,Agua por todas partes.,
```

## Input
```input
locale es
s
```

## Result
```result
START
-> El Mercado
Llegas al mercado.
-> Los Muelles
Agua por todas partes.
END
```

## Expected Translations
```es
id,line,original,translation,status
a798d7a3,6,The Market,El Mercado,
fa2156dc,7,You arrive at the market.,Llegas al mercado.,
4b61a71c,10,The Docks,Los Muelles,
8cad5e19,11,Water everywhere.,Agua por todas partes.,
```
