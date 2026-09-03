# Error: Duplicate Locale in Locales List

Listing the same locale code twice in a single `locales:` line is an error;
locale codes must be unique within the list. This is caught at parse time,
before the story ever runs.

## Script
```cuentitos
--- locales
default: en
locales: en, es, en
---

Hello world. [greeting]
```

## Input
```input
s
```

## Result
```result
duplicate-locale-in-locales-list.cuentitos:3: ERROR: Duplicate locale 'en' in locales list.
```
