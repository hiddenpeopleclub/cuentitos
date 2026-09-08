# The Default Locale Has to Appear in the Locales List

`locales` enumerates every locale the story ships in, and the Default
Locale is one of them. Declaring a default outside the list is a parse-time
error.

## ADRs

- [I18n via Compile-Time Translation Tables](../../../docs/architecture/000017-i18n-translation-tables.md)

## Pending

Blocked on the Compiler milestone. See ADR 000017.

## Script
```cuentitos
--- locales
default: fr
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
default-not-in-locales-list.cuentitos:2: ERROR: Default locale 'fr' is not in the locales list: en, es.
```
