# Reordering Lines Preserves Their Translations

Two lines swap places. Both rows match by Translation Id, so both
translations follow their own text and only the `line` column changes. The
build stays complete and the story runs.

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

Goodbye.
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
Adiós.
Hola.
END
```

## Expected Translations
```es
id,line,original,translation,status
2b687db6,6,Goodbye.,Adiós.,
2d8bd7d9,7,Hello.,Hola.,
```
