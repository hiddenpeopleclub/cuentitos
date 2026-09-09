# Restoring a Deleted Line Revives Its Obsolete Row

"Goodbye." was deleted at some point and its row was kept as obsolete.
The author puts the line back. The row matches by Translation Id in pass 1, so
its `status` clears, its `line` is corrected, and the old translation is live
again.

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
2b687db6,8,Goodbye.,Adiós.,obsolete
```

## Input
```input
locale es
s
```

## Result
```result
START
Hola.
Adiós.
END
```

## Expected Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
2b687db6,7,Goodbye.,Adiós.,
```
