---
name: compatibility-tests
description: Author and run cuentitos compatibility tests. Use when creating new spec tests, debugging existing tests, or running the compatibility suite. Compatibility tests are the source of truth for engine behavior — they drive TDD for every language feature.
---

# Cuentitos Compatibility Tests

Compatibility tests are markdown files that define cuentitos engine behavior
end-to-end. Each test specifies a script, the inputs to drive it, and the
exact output the CLI must produce. The compat runner replays each script
against the built `cuentitos` binary and diffs the output against the
expected `Result` block.

**These tests are the source of truth.** When implementing a feature, write
the failing tests first, then make them pass.

## Layout

```
compatibility-tests/
  <feature-area>/        # e.g. variables-integer, sections, options
    feature/             # happy-path behavior tests
    errors/              # parse-time and runtime error tests
    edge-cases/          # boundary conditions, unusual valid inputs
    cli/                 # CLI-driven concerns (debug `?`, navigation)
```

Not every feature area uses every bucket — create only the buckets you
need. Existing simple features (e.g. `comments`, `goto-end`) live as flat
markdown files directly under `compatibility-tests/<area>/`.

## Filenames

Descriptive kebab-case, no numeric prefix. Examples:
- `arithmetic-on-rhs.md`
- `division-by-zero.md`
- `gating-block-with-children.md`
- `debug-after-load.md`

The filename should read as a sentence fragment describing what the test
verifies. Avoid generic names like `test-1.md`.

## File format

Every test is a markdown file with this exact structure:

````markdown
# <Title>

<One- or two-sentence description of what this verifies and why.>

## Script
```cuentitos
<the cuentitos script>
```

## Input
```input
<comma-separated CLI commands>
```

## Result
```result
<exact expected stdout, line by line>
```
````

The compat runner reads only the three fenced blocks (`cuentitos`, `input`,
`result`). Title and description are for humans.

## Input commands

Comma-separated, no spaces. Each command is a single character:

| Command | Effect |
|---------|--------|
| `n`     | Next block — advance one visible block. |
| `s`     | Skip — run to end, printing every block traversed. |
| `?`     | Debug — print current values of all declared variables. |
| `q`     | Quit. |

Default to `s` for tests that just verify final output. Use `n` when block-
by-block stepping matters. Use `?` for variable-state assertions in `cli/`
tests.

## Output format

The CLI brackets every successful run with `START` and `END`:

```result
START
First line.
Second line.
END
```

- Each shown block emits its raw text on one line.
- Gated `req` blocks that fail their condition do not appear.
- `?` output appears as `<name>: <value>` lines, one per declared variable,
  in declaration order, at the position the `?` was issued.

### Error output

Parse-time and runtime errors do **not** get `START`/`END`. The result is
just the error line(s):

```result
<filename>.cuentitos:<line>: ERROR: <message>
```

The filename in the error message is derived from the test filename (the
runner copies the script to a temp file named `<test-stem>.cuentitos`).
Always include the `:<line>:` and `ERROR:` prefix. Match the wording style
of existing error tests in the same feature area.

## Conventions

### One outcome per test
Each test should verify a single behavior. If two scripts demonstrate
different outcomes (e.g. precedence with vs. without parens), write two
files. Don't pack multiple unrelated assertions into one `Result`.

### Match existing wording
Error messages are part of the spec. When adding a new error case, search
the existing tests in the same `<feature-area>/errors/` folder and reuse
phrasing where the error category overlaps. New error wording should be
agreed before tests land — once a string is in a `.md` Result block, it's
locked.

### Indentation matters
Cuentitos uses **2 spaces per level** for parent-child relationships. Use
spaces, not tabs. Mis-indented scripts will produce `InvalidIndentation`
parse errors.

### Variables block
Variable declarations live in a `--- variables ... ---` fence at the top
of the script. Default values are evaluated at parse time, so any error in
a default (overflow, division by zero, undefined identifier) is a
parse-time error.

## Running tests

```bash
# Build and run the entire suite
./bin/run-compat

# Run a single test
./bin/run-compat compatibility-tests/variables-integer/feature/arithmetic-on-rhs.md

# Run all tests in a feature area (manual glob)
./bin/run-compat 'compatibility-tests/variables-integer/**/*.md'
```

The runner prints `PASS` or `FAIL` per test, with diffs for failures. A
non-zero exit code means at least one test failed.

Behind the scenes the runner builds with `cargo build`, then invokes
`cargo run --bin cuentitos-compat -- ./target/debug/cuentitos <glob>`.

## TDD workflow

1. **Write the failing test.** Author a new `.md` file in the appropriate
   bucket. Run `./bin/run-compat <path>` and confirm it fails. The failure
   message tells you whether the parser or runtime needs work.
2. **Land the test on its own.** When a feature is being split between a
   "compatibility tests" task and an "implementation" task (the standard
   pattern in this repo), the tests PR ships first with all new tests
   failing. Reviewers vet the spec; the implementation PR makes the tests
   pass.
3. **Implement.** Make the tests pass without modifying them. If a test
   needs to change during implementation, treat it as a spec change and
   call it out explicitly in the PR.
4. **Verify.** Before marking the work done:
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
   - `./bin/run-compat`

## Authoring checklist

When adding a new test:
- [ ] Filename is descriptive kebab-case, no numbering.
- [ ] Lives in the right bucket (`feature/`, `errors/`, `edge-cases/`,
      `cli/`) under the right feature area.
- [ ] Title is a sentence fragment describing the behavior.
- [ ] Description explains why this case matters (1–2 sentences).
- [ ] Script uses 2-space indentation, no tabs.
- [ ] `Input` block uses comma-separated commands.
- [ ] `Result` block has `START`/`END` for normal output, or just the
      error line(s) for error tests.
- [ ] Verifies a single outcome — split scripts with multiple outcomes
      across separate files.
- [ ] Error wording matches the convention used elsewhere in the same
      `errors/` folder.
- [ ] Test fails before the feature is implemented (TDD); test passes
      after.
