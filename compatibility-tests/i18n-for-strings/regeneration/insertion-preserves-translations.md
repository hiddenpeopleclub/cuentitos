# Inserting a Line Preserves the Translations Below It

The author adds a line at the top of the script. Every row below it
matches by Translation Id, keeps its translation, and has its `line` corrected.
The inserted line gets a fresh row with an empty translation.

This is the case version 0.2 got wrong. Line-number ids re-keyed every
translation below the insertion point and silently attached each one to the
wrong text.

Pass 1 runs to completion before pass 2 begins. The inserted line sits at line
6, and the row previously recorded at line 6 holds "Hello.", but that row is
already claimed by id, so no line match is available to it.

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

A new opening line.
Hello.
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
s
```

## Result
```result
insertion-preserves-translations.cuentitos:6: ERROR: Missing 'es' translation for 954bbbe4: "A new opening line."
```

## Expected Translations
```es
id,line,original,translation,status
954bbbe4,6,A new opening line.,,
2d8bd7d9,7,Hello.,Hola.,
2b687db6,8,Goodbye.,Adiós.,
```
