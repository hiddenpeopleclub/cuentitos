# An Edited Line Carries Its Translation Over and Is Marked for Review

The author edits line 6 from "Hello." to "Hello there.". The new text
has a new Translation Id, so pass 1 finds nothing. Pass 2 matches the leftover
row by line number, carries the translation over, and sets `status` to
`review` so the translator knows to look at it.

The carried-over translation is live: running under `es` renders it.

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

Hello there.
Goodbye.
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
Adiós.
END
```

## Expected Translations
```es
id,line,original,translation,status
23ea498e,6,Hello there.,Hola.,review
2b687db6,7,Goodbye.,Adiós.,
```
