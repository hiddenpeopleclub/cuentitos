# Cuentitos

A probabilistic narrative environment: a language for writing interactive
stories, and the parser, compiler and runtime that execute them.

This glossary grows as terms are resolved. It currently covers the
internationalization vocabulary.

## Language

### Story structure

**Script**:
The `.cuentitos` source file a writer authors.

**Block**:
A single addressable element of a story, such as a line of text, a section,
or a jump.

**Section Display Name**:
The human-readable title of a section, shown to players and translatable.

**Section Identifier**:
The name a `->` jump resolves against, always held in the Default Locale.
_Avoid_: section name, section path

**Frontmatter Block**:
A `--- <name>` declaration at the head of a script that configures the story,
such as `--- variables` or `--- locales`.

**Database**:
The parsed representation of a script that the runtime executes.

**Compiled Database**:
The serialized artifact `compile` emits, carrying the blocks together with
the Translation Table for every Locale.

### Internationalization

**Locale**:
A language a story can be rendered in, named by a BCP 47 code such as `es`.

**Default Locale**:
The locale the script's own text is written in, declared in the `--- locales`
frontmatter block.
_Avoid_: source language, base language, original language

**Translation Id**:
The stable key identifying one piece of translatable text, derived from that
text's content and assigned by the parser.
_Avoid_: i18n id, message id, string key, translation key

**Translation File**:
The per-locale CSV under `locales/` holding one row per Translation Id, with
columns `id`, `line`, `original` and `translation`.
_Avoid_: locale file, language file, string table

**Original**:
The Default Locale text a row was translated from, refreshed on every
Regeneration.
_Avoid_: source text, msgid

**Regeneration**:
The compile-time pass that rewrites every Translation File from the current
script, preserving the translations already recorded.
_Avoid_: extraction, sync, merge

**Translation Table**:
The per-Locale map from Translation Id to translated text that compile folds
into the Compiled Database, and the only source the runtime reads.

**Obsolete Row**:
A Translation File row whose Translation Id no longer appears in the script,
retained and marked so its translation can be salvaged.

**Missing Translation**:
A Translation File row with an empty translation cell, which fails the
build.

## Relationships

- A **Script** declares one **Default Locale** and zero or more additional
  **Locales**
- Each non-default **Locale** has one **Translation File**
- A **Translation File** holds one row per **Translation Id**, plus any
  **Obsolete Rows**
- A **Translation Id** is derived from the **Original** text, so two blocks
  with identical text share one id and one translation
- **Regeneration** matches existing rows to the current script by
  **Translation Id** first, then by line number
- **String** blocks, **Option** blocks and **Section Display Names** each
  carry a **Translation Id**; a **Section Identifier** carries none
- Compile folds every **Translation File** into a **Translation Table** inside
  the **Compiled Database**, so the runtime reads no CSV

## Example dialogue

> **Dev:** "If the writer fixes a typo in a line, does its **Translation Id**
> stay the same?"
>
> **Domain expert:** "It changes — the id comes from the text. The old row
> becomes an **Obsolete Row** and a fresh one appears."
>
> **Dev:** "So the Spanish translation is lost?"
>
> **Domain expert:** "**Regeneration** finds it by line number and carries it
> over, marked for review. If it can't, the **Obsolete Row** keeps it around
> for the translator to salvage."

## Flagged ambiguities

- "id" was used for both the **Translation Id** and the runtime's internal
  string index — resolved: these are distinct, and only the former appears in
  a **Translation File**.
- "default" was read as "fallback locale" — resolved: the **Default Locale**
  is the language the script is written in.
- **Missing Translation** was read as a runtime rendering — resolved: it is a
  compile-time failure, so the runtime only ever sees complete
  **Translation Tables**.
- "section name" was used for both the **Section Display Name** and the
  **Section Identifier** — resolved: these are distinct, and the parser stores
  them as separate strings so translating one leaves jumps intact.
