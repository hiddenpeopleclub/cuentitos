# Default Locale Renders Untouched

This test establishes the inline syntax for string i18n end to end, since no
such syntax exists in the language yet. Three new constructs and one new CLI
command make up the whole feature surface, and every other test in
`compatibility-tests/i18n-for-strings/` builds on this same grammar.

**Locale config block** — a `--- locales ... ---` fence at the top of the
script, styled after `--- variables ... ---`, declares the full set of
locales and which one is the default:

```
--- locales
default: en
locales: en, es
---
```

`default:` names exactly one locale code. `locales:` lists every locale the
script supports, comma-separated, and must include the default locale. Both
lines are required and may appear in either order.

**Per-block i18n key** — a text block gets a stable, author-chosen key by
ending its source line with a bracketed identifier:

```
Hello world. [greeting]
```

The parser strips the trailing ` [key]` before treating the remainder as
narrative text. The key becomes available to `--- translation ... ---`
blocks later in the same script. A text line with no bracketed key has no
i18n key, and it renders its default-locale text verbatim in every locale.

**Per-locale translation block** — a `--- translation <locale> ... ---`
fence, one per non-default locale, supplies the translated text for each
key:

```
--- translation es
greeting: "Hola mundo."
---
```

The header names a locale that must appear in the `locales:` list. Each
body line has the shape `<key>: "<translated text>"`. A locale with no
`--- translation ... ---` block anywhere in the script has zero
translations. A key with no entry in a given locale's translation block has
no translation in that locale.

**Locale-switch CLI command** — `locale <code>` selects the active locale
for the rest of the run, modeled on `seed <N>`: silent on success, and a
`WARNING:` line (not an `ERROR:`) when `<code>` is not one of the declared
locales, since switching locale is a runtime input concern rather than a
script defect. The active locale starts as the declared default and stays
there until a successful `locale <code>` switch changes it.

This test covers the simplest case: no `locale` switch happens at all, so
the story renders in the default locale, and the rendered text matches the
source exactly, with the bracketed key stripped.

## Script
```cuentitos
--- locales
default: en
locales: en, es
---

Hello world. [greeting]
Goodbye. [farewell]
```

## Input
```input
s
```

## Result
```result
START
Hello world.
Goodbye.
END
```
