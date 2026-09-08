# Regenerating an Up-to-Date File Changes Nothing

A Translation File already matching the script regenerates byte for
byte. A `review` mark survives, because it is cleared by the translator rather
than by the compiler. An obsolete row stays obsolete as long as its text is
absent from the script.

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
2b687db6,7,Goodbye.,Adiós.,review
6139c5b1,9,Old line.,Vieja línea.,obsolete
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
2b687db6,7,Goodbye.,Adiós.,review
6139c5b1,9,Old line.,Vieja línea.,obsolete
```
