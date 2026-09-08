# Switching to an Undeclared Locale Is a Runtime Error

`locale fr` names a locale the script never declared. The CLI reports
it and leaves the active locale alone, so the rest of the run continues in the
Default Locale.

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
```

## Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
```

## Input
```input
locale fr
s
```

## Result
```result
START
ERROR: Unknown locale: 'fr'. Declared locales: en, es.
Hello.
END
```

## Expected Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
```
