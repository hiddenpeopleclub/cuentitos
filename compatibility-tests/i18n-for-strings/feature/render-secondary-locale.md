# Render in a Secondary Locale

`locale es` switches the active locale. Every translatable block then
renders from the `es` Translation Table baked into the compiled database. The
script is identical to the default-locale test.

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
2d8bd7d9,6,Hello.,Hola.,
2b687db6,7,Goodbye.,Adiós.,
```
