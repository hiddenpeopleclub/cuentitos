---
created: 2026-08-29
---

# I18n for Strings — implementation

Implement string i18n to satisfy `compatibility-tests/i18n-for-strings/`.

**Prerequisite:** `i18n-for-strings-compatibility-tests` done.

## Touch points

- `common/src/i18n.rs` — `I18n` struct, `LanguageDb`, locale config.
- `parser/src/` — assign i18n IDs to text blocks; parse locale config and
  translation files.
- `runtime/src/lib.rs` — render text in the requested locale; surface
  missing translations.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
