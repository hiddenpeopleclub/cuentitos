# A Declared Locale With No File Gets One Generated

`fr` is declared and `locales/fr.csv` does not exist. Regeneration
creates it, filled with every translatable line and no translations, which is
the translator's starting worklist. The build then fails on the first Missing
Translation, so the file exists but no database is emitted.

## ADRs

- [I18n via Compile-Time Translation Tables](../../../docs/architecture/000017-i18n-translation-tables.md)

## Pending

Blocked on the Compiler milestone. See ADR 000017.

## Script
```cuentitos
--- locales
default: en
locales: en, fr
---

Hello.
```

## Input
```input
s
```

## Result
```result
declared-locale-without-file.cuentitos:6: ERROR: Missing 'fr' translation for 2d8bd7d9: "Hello."
```

## Expected Translations
```fr
id,line,original,translation,status
2d8bd7d9,6,Hello.,,
```
