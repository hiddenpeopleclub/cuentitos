# Error: Default Locale Not Declared

The `default:` locale in a `--- locales ... ---` block must also appear in
that same block's `locales:` list. Naming a default that is not in the list
is caught at parse time, before the story ever runs.

## Script
```cuentitos
--- locales
default: fr
locales: en, es
---

Hello world. [greeting]
```

## Input
```input
s
```

## Result
```result
default-locale-not-declared.cuentitos:2: ERROR: Invalid default locale: 'fr' is not declared in locales.
```
