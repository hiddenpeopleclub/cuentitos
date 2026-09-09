---
created: 2026-08-29
---

# I18n for Strings — implementation

Implement string i18n to satisfy `compatibility-tests/i18n-for-strings/`.

**Prerequisites:** `compiler-script-to-json`, `compiler-script-to-binary`,
and `i18n-for-strings-compatibility-tests`.

**Design:** [ADR 000017](../../docs/architecture/000017-i18n-translation-tables.md).

## Touch points

- `parser/src/parser.rs` — generalize the leading-frontmatter scan from the
  single hardcoded `--- variables` case (around line 1413) to consuming known
  directive blocks in any order, and add `--- locales`.
- `parser/src/parsers/locales_parser.rs` — new, mirroring
  `variables_parser.rs`.
- `parser/src/parser.rs` (around line 1621) — stop sharing `name_string_id`
  as the section's implicit `id_string_id`. Allocate a separate string so a
  translated display name cannot affect jump resolution.
- `common/src/i18n.rs` — new: locale declarations, Translation Tables, and
  the id→line association Regeneration matches on.
- `common/src/database.rs` — carry the Translation Tables so they serialize
  into the compiled database.
- `compiler/src/i18n.rs` — Regeneration and the CSV read/write. Two-pass
  match: id, then line. Retain Obsolete Rows under a `status` of `obsolete`,
  after the live rows. Write live rows in
  script order.
- `compiler/` — fail the build on any Missing Translation, reporting each at
  its source location, and emit no database.
- `runtime/` — a current locale, locale switching, and rendering from the
  baked Translation Table.
- `cli/src/main.rs` — a `locale <code>` input command, following the existing
  `seed <n>` precedent around line 91.

## Notes

- `Block` already carries `line` (`common/src/block.rs:27`); the id→line
  association is what needs to reach the Translation File.
- Regeneration runs only under compile. `run` performs none, so running a
  story never mutates the author's source tree.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
