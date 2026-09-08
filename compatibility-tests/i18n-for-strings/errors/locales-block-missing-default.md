# A Locales Block Without a Default Is a Parse Error

The Default Locale is the language the script itself is written in, so
a `--- locales` block has to name it. There is no fallback to the first entry
of the list.

## ADRs

- [I18n via Compile-Time Translation Tables](../../../docs/architecture/000017-i18n-translation-tables.md)

## Pending

Blocked on the Compiler milestone. See ADR 000017.

## Script
```cuentitos
--- locales
locales: en, es
---

Hello.
```

## Input
```input
s
```

## Result
```result
locales-block-missing-default.cuentitos:1: ERROR: Locales block is missing 'default'.
```
