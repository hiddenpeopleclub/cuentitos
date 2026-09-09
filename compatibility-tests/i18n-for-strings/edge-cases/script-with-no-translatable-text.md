# A Script With No Translatable Text Regenerates an Empty File

Comments and frontmatter produce no Translation Ids. The Translation
File is still written, carrying its header and no rows, and the build
succeeds because there is nothing left untranslated.

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

// Nothing to say yet.
```

## Translations
```es
id,line,original,translation,status
```

## Input
```input
locale es
s
```

## Result
```result
START
END
```

## Expected Translations
```es
id,line,original,translation,status
```
