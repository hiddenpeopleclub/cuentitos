# A Missing Translation Fails the Build

A row with an empty translation stops compilation and emits no
database. Declaring a locale is a statement that the locale is complete; an
author whose translation lags behind leaves the locale undeclared.

The failure does not depend on the active locale. The story below would render
in `en`, and it still refuses to build.

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

Hello.
Goodbye.
```

## Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
2b687db6,7,Goodbye.,,
```

## Input
```input
s
```

## Result
```result
missing-translation-fails-build.cuentitos:7: ERROR: Missing 'es' translation for 2b687db6: "Goodbye."
```

## Expected Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
2b687db6,7,Goodbye.,,
```
