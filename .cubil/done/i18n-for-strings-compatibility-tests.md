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
`locales/<code>.csv` with columns `id, line, original, translation, status`.

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
- `## Expected Translations` — what Regeneration must produce, compared byte
  for byte against what the run leaves under `locales/`.

Extending `TestCase` and `TestRunner` to handle both is part of this task.

## What to cover

`feature/`
- Render a story in the Default Locale.
- Render the same story in a secondary locale.
- Translated section display names render, while `->` jumps to the same
  sections keep resolving.
- Option text translates.

`regeneration/`
- Inserting a line at the top keeps every translation below it, corrects their
  line numbers, and adds an empty row for the new line. This is the case
  version 0.2 got wrong.
- Reordering two lines keeps each translation with its own text.
- An edited line carries its translation over by line match, with `status` set
  to `review`.
- A deleted line is retained with `status` set to `obsolete`, after the live
  rows.
- Restoring a deleted line revives its obsolete row and clears the status.
- Regeneration is idempotent: an up-to-date file, `review` and `obsolete` marks
  included, regenerates byte for byte.

`errors/`
- A Missing Translation fails the build, reports
  `<file>:<line>: ERROR: Missing '<locale>' translation for <id>: "<original>"`,
  and emits no database.
- A locale requested at runtime that the frontmatter never declared.
- A malformed `--- locales` block, and one missing its `default:`.
- A `default:` absent from the `locales:` list.

`edge-cases/`
- Two blocks with identical text share one id and one translation.
- A declared locale whose CSV does not exist yet gets one generated.
- Text containing commas and quotes, round-tripping through CSV.
- A script with no translatable text regenerates a header-only file.

Every test is marked `## Pending` until the Compiler milestone lands, so the
suite stays green while they sit there as the specification.

## Reference

- `docs/architecture/000017-i18n-translation-tables.md`
- Version 0.2: `common/src/i18n.rs`, `compiler/src/i18n.rs`,
  `palabritas/src/parser.rs` at tag `v0.2.2`. Note that `I18n::process`
  there inserts rows only when both CSV fields are empty, so it never loaded
  a translation. Do not port that condition.
