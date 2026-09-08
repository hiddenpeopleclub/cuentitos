# Text Containing Commas and Quotes Round-Trips Through the CSV

`original` and `translation` hold arbitrary story text. Fields
containing a comma or a double quote are quoted, and embedded double quotes
are doubled, following RFC 4180. Regeneration reproduces the same quoting.

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

Wait, what?
She said "hello", then left.
```

## Translations
```es
id,line,original,translation,status
3fe2a076,6,"Wait, what?","¿Espera, qué?",
1e28b558,7,"She said ""hello"", then left.","Dijo ""hola"" y se fue.",
```

## Input
```input
locale es
s
```

## Result
```result
START
¿Espera, qué?
Dijo "hola" y se fue.
END
```

## Expected Translations
```es
id,line,original,translation,status
3fe2a076,6,"Wait, what?","¿Espera, qué?",
1e28b558,7,"She said ""hello"", then left.","Dijo ""hola"" y se fue.",
```
