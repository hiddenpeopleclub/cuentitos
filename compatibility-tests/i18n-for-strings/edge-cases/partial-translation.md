# Partial Translation Mixes Translated and Missing Keys

A secondary locale's `--- translation ... ---` block can cover only some of
the default locale's keys. Keys with an entry render translated; keys with
no entry render `MISSING TRANSLATION <key>`, in the order the blocks appear
in the story, regardless of the order translations were declared.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

--- translation es
greeting: "Hola."
thanks: "Gracias."
---

Hello. [greeting]
Please wait. [wait]
Thank you. [thanks]
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
MISSING TRANSLATION wait
Gracias.
END
```
