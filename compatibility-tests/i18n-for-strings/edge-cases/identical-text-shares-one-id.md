# Identical Text Shares a Single Translation Id

Translation Ids come from content, so two lines reading "Wait." share
one id and therefore one row. The row records the line of the first
occurrence, and both lines render from it. Translating the two occurrences
differently is not expressible; context disambiguation is deferred.

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

Wait.
Something happens.
Wait.
```

## Translations
```es
id,line,original,translation,status
b0c879ab,6,Wait.,Espera.,
58649026,7,Something happens.,Algo pasa.,
```

## Input
```input
locale es
s
```

## Result
```result
START
Espera.
Algo pasa.
Espera.
END
```

## Expected Translations
```es
id,line,original,translation,status
b0c879ab,6,Wait.,Espera.,
58649026,7,Something happens.,Algo pasa.,
```
