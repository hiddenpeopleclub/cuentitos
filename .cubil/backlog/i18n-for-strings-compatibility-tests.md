---
created: 2026-08-29
---

# I18n for Strings — compatibility tests

Write the compat tests for string i18n **before** the implementation lands.
Sibling task: `i18n-for-strings-implementation`. TDD: tests first.

**Prerequisite:** `compiler-script-to-json` and `compiler-script-to-binary`.
The feature is defined in terms of a compile step, so these tests cannot run
until one exists.

**Design:** [ADR 000017](../../docs/architecture/000017-i18n-translation-tables.md).
Read it first. An earlier attempt at these tests was rejected for inventing
in-script translation blocks and author-written keys; both are ruled out.

## Feature summary

Locales are declared in a `--- locales` frontmatter block. The parser derives
a Translation Id from each translatable block's Default Locale text, so
authors write no keys. Each non-default locale owns
`locales/<code>.csv` with columns `id, line, original, translation`.

Compile regenerates those files from the current script, matching existing
rows by Translation Id and then by line number, retaining unmatched rows as
marked Obsolete Rows. It then folds every file into a Translation Table inside
the compiled database, which is the only thing the runtime reads. A row with
an empty translation fails the build.

## Test format extensions

These tests need two new sections, specified in
`docs/compatibility-test-format.md`:

- `## Translations` — pre-existing Translation Files, one fenced block per
  locale, materialized by the runner under `locales/`.
- `## Expected Translations` — what Regeneration must produce. A test
  covering only the merge needs no `## Input` or `## Result`.

Extending `TestCase` and `TestRunner` to handle both is part of this task.

## What to cover

`feature/`
- Render a story in the Default Locale.
- Render the same story in a secondary locale.
- Translated section display names render, while `->` jumps to the same
  sections keep resolving.
- Option text translates.

`regeneration/`
- A new line adds a row with an empty translation, in script order.
- An unchanged line keeps its translation when other lines are inserted above
  it, proving id matching survives line shifts.
- An edited line carries its translation over by line match, marked for
  review.
- A deleted line becomes a marked Obsolete Row.
- Regeneration is idempotent: running it twice produces identical files.

`errors/`
- A Missing Translation fails the build, reports
  `<file>:<line>: ERROR: Missing \`<locale>\` translation for <id>: "<original>"`,
  and emits no database.
- A locale requested at runtime that the frontmatter never declared.
- A malformed `--- locales` block, and one missing its `default:`.
- A `default:` absent from the `locales:` list.

`edge-cases/`
- Two blocks with identical text share one id and one translation.
- A declared locale whose CSV does not exist yet.
- A Translation File carrying rows for ids absent from the script.
- Text containing commas and quotes, round-tripping through CSV.

## Reference

- `docs/architecture/000017-i18n-translation-tables.md`
- Version 0.2: `common/src/i18n.rs`, `compiler/src/i18n.rs`,
  `palabritas/src/parser.rs` at tag `v0.2.2`. Note that `I18n::process`
  there inserts rows only when both CSV fields are empty, so it never loaded
  a translation. Do not port that condition.
