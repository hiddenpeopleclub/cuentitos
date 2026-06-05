# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cuentitos` is a probabilistic narrative environment designed to make creating interactive stories frictionless. This is a Rust workspace implementing the core language, parser, compiler, and runtimes.

**IMPORTANT**: The project is currently being reimplemented (v0.3). Expect instability.

## Common Commands

### Building
```bash
cargo build
```

### Running Tests
```bash
# Run all tests
cargo test

# Run tests for a specific package
cargo test -p cuentitos-parser

# Run a specific test
cargo test test_name
```

### Running the CLI
```bash
# Build and run a script
cargo run --bin cuentitos run <script_path> <input_string>

# Example with inputs (n=next, s=skip, q=quit)
cargo run --bin cuentitos run ./test.cuentitos "n,n,s"
```

### Running Compatibility Tests
```bash
# Build and run all compatibility tests
./bin/run-compat

# Or manually:
cargo build
cargo run --bin cuentitos-compat -- ./target/debug/cuentitos ./compatibility-tests/**/*.md
```

## Development Workflow

**Test-Driven Development (TDD)**: This project uses TDD with compatibility tests.

1. Write a compatibility test in `compatibility-tests/` following the format in existing tests
2. Run `./bin/run-compat` to see it fail
3. Implement the feature
4. Run `./bin/run-compat` again to see it pass

Compatibility tests define how the engine should work - they are the source of truth.

## Task Bookkeeping

Tasks live as Markdown files in `.cubil/` (`backlog/`, `doing/`, `done/`). When you are picking up a task:

1. Run `cubil start <slug>` before you begin work (moves it from `backlog/` to `doing/`).
2. Run `cubil finish <slug>` when the task is complete and merged/pushed (moves it from `doing/` to `done/`).
3. Commit both transitions — the file moves are tracked in git.

This applies to any agent (local or remote) acting on a roadmapped task. The roadmap (`.cubil/roadmaps/`) renders status by reading those folders, so keeping them current is what drives `cubil roadmap next` and `cubil roadmap show`.

## Architecture

### Workspace Structure

This is a Rust workspace with these crates:

- **`common/`**: Shared types and data structures (`Block`, `Database`, `BlockType`)
- **`parser/`**: Language parser that transforms cuentitos scripts into a `Database`
- **`cli/`**: Main CLI tool (`cuentitos`) for running scripts
- **`runtime/`**: Runtime executor for parsed scripts
- **`compat/`**: Compatibility test runner (`cuentitos-compat`)

### Core Data Model

The parser produces a `Database` containing:
- **Blocks**: Tree structure of narrative blocks with parent-child relationships
- **Strings**: Content strings referenced by blocks

Each `Block` has:
- `block_type`: `Start`, `String(StringId)`, `Section{id, display_name}`, `GoToSection{path, target_block_id}`, or `End`
- `parent_id`: Optional reference to parent block
- `children`: Vector of child block IDs
- `level`: Indentation level (0-based, 2 spaces per level)

Blocks form a hierarchy where indentation determines parent-child relationships.

### Parser Design

The parser is line-based and extensible:
1. Goes through the script line by line
2. Asks registered block parsers if they can parse the line
3. First matching parser handles the line and returns appropriate blocks
4. Blocks are added to the database with parent-child relationships established

This design allows easy extension through new block parsers (see `parser/src/block_parsers/`).

### CLI Input Commands

Runtime commands during script execution:
- `n`: Move to next block
- `s`: Skip to end, showing all intermediate blocks
- `q`: Quit

### Compatibility Test Format

Tests are markdown files with this structure:
```markdown
# Test Name

Description

## Script
```cuentitos
// cuentitos script
```

## Input
```input
n,n,s
```

## Result
```result
// expected output
```
```

The test runner compares CLI output against the Result section.

## Key Design Principles

- **Write clear, idiomatic Rust code**
- **Use expressive variable names** (e.g., `is_ready`, `has_data`)
- **Use descriptive names, no abbreviations** — prefer `Expression` over `Expr`, `BinaryOperator` over `BinOp`, `RequirementStatement` over `ReqStatement`, `evaluate_expression` over `eval_expr`. Applies to types, functions, fields, locals, and module names alike. Domain words (`req`, `set`) used as language keywords stay short in source syntax, but their Rust identifiers should be spelled out.
- **Follow Rust naming conventions**: snake_case for functions/variables, PascalCase for types
- **Embrace ownership and the type system**
- **Avoid code duplication**
- **Write Rust unit tests for parser logic and validation rules** - Don't rely solely on compatibility tests
- **Refer to ADRs in `docs/architecture/`** for design rationale

## Important Notes

- Indentation is **2 spaces per level** (enforced by parser)
- Parser validates indentation and returns `InvalidIndentation` errors when violated
- The parser expects blocks to form a valid hierarchy (can't skip levels)
- All strings are stored in the database and referenced by ID to avoid duplication