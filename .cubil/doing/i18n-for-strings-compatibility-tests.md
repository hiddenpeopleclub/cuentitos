---
created: 2026-08-29
---

# I18n for Strings — compatibility tests

Write the compat tests for string i18n **before** the implementation lands.
Sibling task: `i18n-for-strings-implementation`. TDD: tests first.

## Feature summary

Support multiple locales for story text. The engine stores string content
keyed by locale, so a single script renders in different languages.

- Locales and the default locale are declared in config.
- At parse time, each text block gets an i18n ID; the default locale stores
  the original text.
- Additional locale files provide translations keyed by the same IDs.
- The runtime renders text in the requested locale; missing translations
  surface as `MISSING TRANSLATION ...`.

## What to cover

- `feature/` — render in the default locale, render in a secondary locale,
  missing-translation fallback.
- `errors/` — undeclared locale, malformed locale config.
- `edge-cases/` — partial translations, locale with no translations.

## Reference

- `common/src/i18n.rs` and `palabritas/src/parser.rs` (version-0.2 branch).
