# Option Text Translates

`Option` blocks carry a Translation Id like any other player-facing
text. The choice list and the selection echo both render in the active
locale.

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

What do you want to do?
  * Go left
    You went left
  * Go right
    You went right
```

## Translations
```es
id,line,original,translation,status
68f2a0c9,6,What do you want to do?,¿Qué quieres hacer?,
2da1a1b9,7,Go left,Ir a la izquierda,
41168c11,8,You went left,Fuiste a la izquierda,
c06e38ff,9,Go right,Ir a la derecha,
15fe61de,10,You went right,Fuiste a la derecha,
```

## Input
```input
locale es
2
s
```

## Result
```result
START
¿Qué quieres hacer?
  1. Ir a la izquierda
  2. Ir a la derecha
> Selected: Ir a la derecha
Fuiste a la derecha
END
```

## Expected Translations
```es
id,line,original,translation,status
68f2a0c9,6,What do you want to do?,¿Qué quieres hacer?,
2da1a1b9,7,Go left,Ir a la izquierda,
41168c11,8,You went left,Fuiste a la izquierda,
c06e38ff,9,Go right,Ir a la derecha,
15fe61de,10,You went right,Fuiste a la derecha,
```
