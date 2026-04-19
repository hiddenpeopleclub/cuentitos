# Compatibility Tests Layout

`compatibility-tests/` is organized by feature, not by chronological test number.
This file is the in-repo source of truth for how tests are arranged and how to add new ones.

## Context

Prior to this layout, all compatibility tests lived flat in `compatibility-tests/`
with an 11-digit prefix encoding the order in which they were written
(e.g. `00000000032-basic-jump-to-sibling.md`). The flat layout made it hard to
find related tests, see what coverage existed for a given feature, and decide
where a new test belonged.

The new layout groups tests by the language feature they exercise and drops the
numeric prefixes entirely. Tests are addressed by
`<feature>/<category>/<name>.md`.

## Layout

```
compatibility-tests/
├── strings/              # plain text lines and indentation basics
├── variables/            # variable declarations and usage
├── sections/             # section definitions and nesting
├── comments/             # `//` comment handling
├── goto-section/         # `-> path` unconditional jumps
├── goto-section-and-back/# `<-> path` call-and-return (see ADR 000014)
├── goto-end/             # `-> END` terminate
├── goto-start/           # `-> START` restart from top, preserving state
├── goto-restart/         # `-> RESTART` restart from top, clearing state
└── options/              # `*` option blocks and player choice
```

Each feature folder has up to four category subdirectories:

- **`feature/`** — happy-path tests showing intended usage.
- **`errors/`** — common errors and the exact error output they produce.
- **`edge-cases/`** — unusual but valid usage. **Warnings live here**, not in
  `errors/`, because a warning means "valid but weird."
- **`cli/`** — CLI-driven tests for this feature (flat — no further split).
  Error-style CLI tests get an `error-` filename prefix.

Not every feature has every category: `strings/` has no `edge-cases/` or `cli/`,
`comments/` has no `errors/` or `cli/`, etc. Add a category folder when the
first test for it shows up; don't pre-create empty ones.

## Classification rules

- **Warnings → `edge-cases/`.** Anything the parser/runtime accepts but
  complains about goes here, not in `errors/`. Examples: unreachable-code
  warnings, `<-> END` warnings, section name whitespace warnings.
- **Generic indentation errors → `strings/errors/`.** Tests like
  tab-indentation and invalid-indentation fire on the most basic script form
  (no sections, just text), so they belong with strings rather than a dedicated
  indentation folder.
- **Section-specific indentation errors → `sections/errors/`.** If the error
  only manifests inside a section context (indentation-jump,
  invalid-section-indentation), it lives with sections.
- **No `indentation/` folder.** Indentation is cross-cutting; classify by the
  construct the test is actually exercising.
- **`cli/` is flat.** Don't create `cli/feature/` and `cli/errors/`
  subdirectories — use an `error-` filename prefix instead
  (e.g. `error-section-not-found.md`).

## Discovery

`bin/run-compat` expands `./compatibility-tests/**/*.md` recursively and runs
every test found. The glob is quoted so bash passes it through literally and
Rust's `glob` crate does the recursive expansion. Adding a new test file
anywhere under `compatibility-tests/` picks it up automatically.

## How to add a new test

1. Decide the **feature** (top-level folder) and **category** (`feature/`,
   `errors/`, `edge-cases/`, or `cli/`).
2. Pick a short descriptive filename: `what-this-test-demonstrates.md`.
   No numeric prefix.
3. Write the test in the standard format:
   ```markdown
   # Test Name

   One-line description.

   ## Script
   ```cuentitos
   // script here
   ```

   ## Input
   ```input
   n,n,s
   ```

   ## Result
   ```result
   expected output here
   ```
   ```
4. If the test's expected output embeds a filename (e.g. an error message like
   `my-test.cuentitos:2: ERROR: ...`), use the **new** filename stem — the
   compat runner builds temp filenames from the test's own basename.
5. Run `./bin/run-compat` and iterate until it passes.

## Rust code that references specific tests

A few Rust unit tests `include_str!` specific compat files. If you move or
rename any of these, update the include paths in:

- `parser/src/parser.rs` — `strings/feature/single-line-and-end.md`
- `runtime/src/lib.rs` — `strings/feature/two-lines-and-end.md`,
  `strings/feature/two-lines-and-skip.md`,
  `strings/feature/nested-strings-with-siblings.md`
- `common/src/test_case.rs` — `strings/feature/single-line-and-end.md`
