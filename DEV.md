# DEV.md

This document provides guidance for AI coding agents working on the
cuentitos project. It defines the development process, workflows, and best
practices to follow.

## Development Flow

The development process follows a structured 7-phase approach:

**Feature Plan → Test Plan → Implement Tests → Implementation Plan → TDD
Development → Verify → Document**

Each phase has specific goals and guidelines detailed below.

---

## Phase 1: Feature Plan

**Goal:** Understand the feature requirements through interactive
discussion.

**Process:**
1. Ask questions **one by one** to understand the feature
2. Start with **high-level questions** first (e.g., "What should comments
   look like?"), then drill down to specifics
3. Ask **as many questions as needed** for full clarity
4. Explore the codebase **anytime needed** to understand context

**Guidelines:**
- Be thorough - it's better to ask too many questions than too few
- Don't make assumptions; always ask when uncertain
- Take notes on requirements as they're discussed

**Output:** Clear understanding of feature requirements and behavior.

---

## Phase 2: Test Plan

**Goal:** Define all compatibility tests needed to verify the feature.

**Process:**
1. **Explore existing compatibility tests** in `compatibility-tests/` to
   understand format and patterns
2. **Propose a list** of test scenarios covering:
   - Happy path cases
   - Edge cases
   - Error cases
   - Integration with existing features
3. **Discuss each test scenario one by one** to refine and confirm

**Guidelines:**
- Understand the test format thoroughly - don't ask about format
  repeatedly
- Ensure tests are comprehensive enough to define "done"
- Consider interactions with existing features

**Output:** Agreed-upon list of test scenarios to implement.

---

## Phase 3: Implement Tests

**Goal:** Write all compatibility tests (which should fail initially).

**Process:**
1. **Write all test files at once** following the compatibility test
   format.
2. **Run `./bin/run-compat`** after creating tests
3. **Show the output** and confirm the failures are correct (Red phase
   of TDD)

**Guidelines:**
- Tests should fail for the right reasons (feature not implemented, not
  syntax errors)
- Place tests in appropriate subdirectories of `compatibility-tests/`
- Follow naming conventions from existing tests

**Output:** Complete set of failing compatibility tests.

---

## Phase 4: Implementation Plan

**Goal:** Design the technical approach and architecture for the
implementation.

**Process:**
1. **Explore the codebase** to understand:
   - Current architecture (e.g., how block parsers work)
   - Existing patterns to follow
   - Where new code should go
2. **Propose the architectural approach** with trade-offs
3. **Ask architectural questions one by one** to discuss decisions
4. **Write a draft ADR** (Architecture Decision Record) documenting:
   - Context and problem
   - Considered options
   - Chosen solution and rationale
   - Consequences

**Guidelines:**
- Follow existing architectural patterns when possible
- Consider extensibility and maintainability
- Document why decisions were made, not just what
- Be conservative - ask when uncertain about architectural choices

**Output:**
- Clear implementation strategy
- Draft ADR in `docs/architecture/`
- Agreement on approach

---

## Phase 5: TDD Development

**Goal:** Implement the feature using Test-Driven Development.

**Process:**
1. Work on **one failing test at a time**:
   - Write minimal code to make it pass (Green)
   - Refactor for quality (Refactor)
   - Move to next test
2. **Run tests after each change** to prevent regression:
   - Run `./bin/run-compat` for compatibility tests
   - Run `cargo test` for unit tests
3. **Commit after logical units** are complete (e.g., a class, module,
   or coherent feature chunk)
4. **Push to origin** automatically after each commit

**Guidelines:**
- Write Rust unit tests for complex logic and validation rules
- Don't rely solely on compatibility tests - add unit tests too
- Keep commits focused and atomic
- If you get stuck, **ask for help** immediately
- If a refactoring breaks existing tests, **stop and ask** before
  proceeding
- Run both compatibility tests and unit tests frequently
- **IMPORTANT: For testing, ONLY use Rust unit tests or compatibility tests. NEVER create external .cuentitos script files for testing purposes.**

**Bug Fixing Protocol:**
When you encounter a bug (either from failing compatibility tests or other sources):
1. **STOP** - Do not try to fix it immediately with compatibility tests
2. **Write a Rust unit test** in the specific module (parser, runtime, etc.) that reproduces the bug
3. **Verify the test fails** for the right reason
4. **Fix the bug** to make the test pass
5. **Verify compatibility tests** now pass as well

This approach:
- Creates focused, fast-running tests for the specific bug
- Makes debugging easier (targeted tests vs. full integration tests)
- Prevents regressions with module-specific test coverage
- Speeds up the development feedback loop

**Commit Messages:**
- Use clear, descriptive messages
- **Do NOT include Claude Code branding** or co-author tags
- Format: Start with a verb (e.g., "Add comment parsing", "Fix
  indentation validation")

**Output:** Working implementation with all tests passing.

---

## Phase 6: Verify

**Goal:** Ensure code quality and catch any issues before completion.

**Process:**
1. **Run the full verification suite:**
   - `cargo test` - All unit tests
   - `./bin/run-compat` - All compatibility tests
   - `cargo clippy` - Linting
   - `cargo fmt` - Formatting
   - `cargo doc` - Build and test Rust docs
2. **Invoke the code review agent:**
   - Use a sub-agent called "cuentitos-reviewer"
   - This agent will perform a thorough code review
3. **Create a code review report** with:
   - Issues found (if any)
   - Suggestions for improvement
   - Code quality observations
4. **Present the report** to the user who will decide what to address

**Guidelines:**
- Don't automatically fix review findings - present them first
- All tests must pass before moving to Document phase
- No clippy warnings should remain
- Code should be properly formatted

**Output:**
- Clean verification run (all tests pass, no warnings)
- Code review report
- Fixes applied per user decision

---

## Phase 7: Document

**Goal:** Create comprehensive documentation for the feature.

**Process:**
1. **Create a documentation plan** covering:
   - ADR finalization
   - CLAUDE.md updates (if needed)
   - Inline code documentation
   - Rust doc comments
   - User manual additions
2. **Ask questions one by one** about documentation decisions
3. For the **user manual** (when it exists):
   - Ask about structure/format desired
   - Provide a plan before writing
   - Ask questions one by one about content
4. **Write all documentation**
5. **Commit documentation separately** from implementation code

**Guidelines:**
- Documentation should be clear and thorough
- Include examples where helpful
- Update CLAUDE.md if the feature changes development patterns
- Ensure Rust docs build without warnings

**Output:** Complete documentation committed separately.

---

## TODO.md Workflow

**How to use TODO.md:**
1. **Mark items as done** `[x]` after full implementation (all phases
   complete)
2. **Add new discovered tasks** as you encounter them during development
3. Items are organized in sections (Compat, Language, Compiler, Plugins)

**Do NOT:**
- Mark items as done before they're truly complete
- Remove items without completing them
- Reorder items arbitrarily

---

## Branching & Pull Requests

**Branch Naming:**
- Format: `v3-todo-item-name`
- Example: `v3-support-comments`
- Use the `./bin/start-feature` script (when available) to select a
  feature from TODO.md and create a branch

**Pull Request Workflow:**
1. **Create PR after Feature Plan phase** with:
   - Description of the feature
   - Draft ADR included
   - Mark as draft if not ready for review
2. **Update PR with each commit:**
   - Commits automatically push to origin
   - PR updates automatically
3. **Mark PR as ready** after Document phase is complete

**Commit Guidelines:**
- Push to origin automatically after each commit
- Keep commits atomic and focused
- **No Claude Code branding** in commit messages
- Commit logical units (classes, modules) together
- Commit documentation separately from implementation

---

## Decision-Making & Autonomy

**When to Ask:**
- Encountering ambiguity or uncertainty (e.g., "Should this error be
  fatal?")
- Stuck during implementation
- A refactoring breaks existing tests
- Architectural decisions
- Documentation structure decisions

**Be Conservative:**
- Ask more questions rather than fewer
- Don't make assumptions
- When in doubt, ask

**No Pre-Approval Required For:**
- Adding dependencies (but document in ADR)
- Changing public APIs (but document in ADR)
- Most implementation decisions (but be ready to explain)

---

## Key Principles

1. **Test-Driven Development** - Write tests first, implement to make
   them pass
2. **Run tests frequently** - After each change to catch regressions
   early
3. **Ask questions one by one** - Don't batch questions; interactive
   discussion works best
4. **Be thorough** - Better to ask too many questions than too few
5. **Document decisions** - ADRs explain why, not just what
6. **Commit logical units** - Keep commits focused and atomic
7. **Explore as needed** - Look at the codebase whenever context is
   needed

---

## Common Commands Reference

```bash
# Build the project
cargo build

# Run all tests
cargo test

# Run tests for a specific package
cargo test -p cuentitos-parser

# Run compatibility tests
./bin/run-compat

# Run linting
cargo clippy

# Format code
cargo fmt

# Build documentation
cargo doc

# Run the CLI
cargo run --bin cuentitos run <script_path> <input_string>

# Start working on a feature (when available)
./bin/start-feature
```

---

## Code Quality Standards

**Must Pass:**
- All unit tests (`cargo test`)
- All compatibility tests (`./bin/run-compat`)
- Clippy with no warnings (`cargo clippy`)
- Formatted code (`cargo fmt`)
- Documentation builds (`cargo doc`)

**Code Style:**
- Write clear, idiomatic Rust code
- Use expressive variable names (e.g., `is_ready`, `has_data`)
- Follow Rust naming conventions: snake_case for functions/variables,
  PascalCase for types
- Embrace ownership and the type system
- Avoid code duplication
- Add meaningful comments for complex logic

---

## Architecture Notes

- Indentation is **2 spaces per level** (enforced by parser)
- Blocks form a hierarchy where indentation determines parent-child
  relationships
- Parser is line-based and extensible through block parsers
- See `docs/architecture/` for ADRs explaining design decisions
- See CLAUDE.md for project overview and architecture details

---

## Notes for AI Agents

- **Don't surprise the user** - ask before taking major actions
- **Explore the codebase freely** - you can read files anytime to
  understand context
- **Use the compatibility tests** - they are the source of truth for
  behavior
- **Follow TDD strictly** - Red (write test) → Green (make it pass) →
  Refactor
- **Be patient and thorough** - quality matters more than speed
