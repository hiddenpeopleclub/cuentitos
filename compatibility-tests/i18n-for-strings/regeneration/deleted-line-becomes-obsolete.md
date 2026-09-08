# A Deleted Line Leaves an Obsolete Row

The author deletes "Goodbye.". Its row matches nothing in either pass,
so it is retained with `status` set to `obsolete` and its translation intact,
ready to be salvaged if the line comes back. Obsolete rows are written after
the live rows and never count as Missing Translations.

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
2b687db6,7,Goodbye.,Adiós.,
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
END
```

## Expected Translations
```es
id,line,original,translation,status
2d8bd7d9,6,Hello.,Hola.,
2b687db6,7,Goodbye.,Adiós.,obsolete
```
