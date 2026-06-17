---
created: 2026-06-17
---

# I18n for Strings

## Summary
Support multiple locales for story text. The engine stores string content keyed by locale, allowing a single script to be rendered in different languages.

Architecture (from version-0.2):
- `I18n` struct: `locales: Vec<LanguageId>`, `default_locale: LanguageId`, `strings: HashMap<LanguageId, LanguageDb>`
- `LanguageDb` is a `HashMap<I18nId, String>` mapping block IDs to translated strings
- Locales and default locale are declared in config (currently `--- variables` block or a config file TBD for v0.3)
- At parse time, each text block gets an i18n ID; the default locale stores the original text
- Additional locale files provide translations keyed by the same IDs
- Runtime renders text in the requested locale; missing translations surface as `MISSING TRANSLATION \`...\` in locale \`...\``

## What to do
Run `/grill-with-docs` first to generate a detailed spec and align on how locale config is declared in v0.3.

Then follow TDD: write compatibility tests first (must fail), then implement.

## Reference
- `common/src/i18n.rs` and `palabritas/src/parser.rs` (version-0.2 branch)
