# A Malformed Entry in the Locales Block Is a Parse Error

Entries in a `--- locales` block are `key: value` pairs. A line
missing its colon is a parse-time error reported at its own line.

## ADRs

- [I18n via Compile-Time Translation Tables](../../../docs/architecture/000017-i18n-translation-tables.md)

## Pending

Blocked on the Compiler milestone. See ADR 000017.

## Script
```cuentitos
--- locales
default en
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
malformed-locales-block.cuentitos:2: ERROR: Invalid locales entry: 'default en'. Expected 'key: value'.
```
