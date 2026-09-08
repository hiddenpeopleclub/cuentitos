# Internationalization of Strings via Compile-Time Translation Tables

### Submitters

- Fran Tufro

## Change Log

- [pending] 2026-09-08

## Context

Stories need to render in multiple languages. Version 0.2 shipped an
implementation of this, and ADR
[000008](000008-i18n-strings.md) described a design that diverged from what
was built. This ADR supersedes 000008 and records the design for version 0.3.

The decision is architecturally significant because it introduces a
compile step that writes to the author's source tree, adds a per-locale
artifact translators own, and constrains how section identifiers are stored.

### What version 0.2 actually did

- `cuentitos.toml` declared `locales` and `default_locale`.
- The parser replaced each `Text` and `Choice` block's content with an id
  derived from the **line number**, filing the original text under
  `i18n.strings[default_locale][id]`.
- `compile()` read `locales/<code>.csv` for each non-default locale, folded
  every locale into `Database.i18n`, and serialized the whole database to
  MessagePack. The runtime read the baked tables and never opened a CSV.
- `get_translation` returned a `MISSING TRANSLATION` placeholder for absent
  keys.

Two problems carried over from that implementation:

1. Line-number ids are unstable. Inserting one line at the top of a script
   re-keyed every translation below it, silently attaching each translation
   to the wrong text.
2. `I18n::process` loaded translations under the condition
   `record.0.is_empty() && record.1.is_empty()`, which inserts only when both
   CSV fields are empty. Version 0.2 therefore loaded zero translations.

## Proposed Design

### Declaration

Locales are declared in a `--- locales` frontmatter block, alongside the
existing `--- variables` block. Both are optional, both must precede story
content, and either order is accepted. `cuentitos.toml` is retired.

```cuentitos
--- locales
default: en
locales: en, es
---
```

### Translation ids

The parser assigns every translatable block a Translation Id derived from the
content of its Default Locale text. Authors never write ids. Two blocks with
identical text share one id and therefore one translation; disambiguation by
context is deferred.

Content-derived ids are stable under reordering and insertion. They change
when the text changes, which correctly marks the translation as stale.

### Translatable blocks

`String` and `Option` blocks produce Translation Ids, as do section display
names. Section jump resolution reads `Section.id` and `Section.id_path`, so
translated display names never affect navigation. To keep that guarantee,
the parser stores a section's implicit identifier as its own database string
rather than sharing the display name's `StringId`, which is what
`parser/src/parser.rs` does today.

### Translation Files

Each non-default locale owns `locales/<code>.csv` with the columns
`id`, `line`, `original`, `translation` and `status`. The `original` column
gives the translator the source text in the same file they work in. The `line`
column records where the text sat at the previous Regeneration. The `status`
column is empty for a current row, `review` for a row whose original text was
edited underneath it, and `obsolete` for a row whose text left the script.
Quoting follows RFC 4180.

### Regeneration

Compile rewrites every Translation File from the current script before doing
anything else, so the translator's worklist is always current. Existing rows
are matched against the script in two passes:

1. **By Translation Id.** The text is unchanged, so the translation carries
   over untouched. Line numbers shifting elsewhere in the file are harmless,
   because this pass matches on content.
2. **By line number,** over whatever remains. A row occupying the same line
   under a different id means the writer edited that line, so the translation
   carries over and is marked for review.

Script entries left unmatched become rows with an empty translation. Rows left
unmatched become Obsolete Rows, retained so their translations can be salvaged;
an obsolete row whose text returns to the script is revived by pass 1, which
clears its status. Live rows are written in script order, obsolete rows after
them.

Pass 1 runs to completion before pass 2 begins. A row is available to a line
match only if no id match has already claimed it.

A `review` mark survives Regeneration, because the translator clears it once
they have looked at the row. Pass 1 preserves whatever a matched row carried,
clearing only `obsolete`.

### Compilation and the runtime

Compile folds every Translation File into the Translation Table for its locale
inside the compiled database, following version 0.2. The runtime reads only
those baked tables.

A row with an empty translation fails the build. Compile reports it at its
source location in the established diagnostic format, following the rest of the
toolchain in stopping at the first diagnostic, and emits no database:

```
story.cuentitos:12: ERROR: Missing 'es' translation for 9b04a1f7: "Hello world."
```

Declaring a locale is therefore a statement that the locale is complete. An
author whose translation lags behind leaves the locale undeclared and keeps
building.

### Compatibility test format

Two sections are added. `## Translations` supplies the pre-existing
Translation Files, which the runner materializes under `locales/`.
`## Expected Translations` asserts what Regeneration must produce, making
carry-over and obsolete marking directly testable. Each takes one fenced block
per locale, keyed by the fence language.

Because `run` performs no Regeneration, the runner compiles a test carrying
either section before running it, and compares the files compile leaves under
`locales/`.

## Considerations

**Hashed ids versus source text as the key.** Using the source text as the key
would make Translation Files hand-writable. The `original` column delivers the
same benefit to translators while keeping ids short and fixed-width, so the id
column stays.

**Losing work on a text edit.** Because ids follow content, editing a line
retires its id. Pass-2 line matching recovers the translation in the common
case, and Obsolete Rows preserve it in the rest.

**Compile writing to the source tree.** Regeneration mutates files the author
owns. It is confined to an explicit compile step for that reason, and `run`
performs no Regeneration.

**A status column over an out-of-band marker.** Marking a row by prefixing it
with a comment line keeps the column set at four, and it stops the file being
CSV, which is the one property that lets translators open it in the tools they
already use. It also cannot express a mark on a row that is still live, which
`review` needs. A fifth column carries both marks and stays parseable.

**Duplicate collapse.** A line needing different translations in different
contexts cannot be expressed. A disambiguator can be added later; doing so
changes ids only for the lines that adopt it.

## Decision

1. Locales are declared in `--- locales` frontmatter. `cuentitos.toml` is
   retired.
2. Translation Ids are content-derived and assigned by the parser.
3. `String` blocks, `Option` blocks, and section display names are
   translatable. Section identifiers are stored separately from display names
   so jumps stay untranslated.
4. Translation Files are per-locale CSVs with `id, line, original,
   translation, status`.
5. Compile regenerates Translation Files, matching by id and then by line, and
   retains Obsolete Rows under a `status` of `obsolete`.
6. Compile folds the Translation Files into Translation Tables inside the
   compiled database. The runtime reads only those.
7. A Missing Translation fails the build and no database is emitted.
8. The compatibility test format gains `## Translations` and
   `## Expected Translations`.

### Sequencing

This design depends on a compile step, which version 0.3 does not yet have.
The I18n milestone moves after `compiler-script-to-json` and
`compiler-script-to-binary` on the roadmap.

### Deferred

- Context disambiguation for duplicate text.
- Fallback chains between related locales.
- Pluralization and interpolation.
- Right-to-left rendering.

## Other Related ADRs

- [I18n Strings](000008-i18n-strings.md) - superseded by this ADR
- [Lines of Text](000005-lines-of-text.md) - the text handling this extends
- [Sections and Navigation](000011-sections-and-navigation.md) - the
  identifier and display name separation jump resolution depends on

## References

- [BCP 47 Language Tags](https://www.rfc-editor.org/info/bcp47)
- [GNU gettext](https://www.gnu.org/software/gettext/) - the extract-and-merge
  workflow Regeneration follows
