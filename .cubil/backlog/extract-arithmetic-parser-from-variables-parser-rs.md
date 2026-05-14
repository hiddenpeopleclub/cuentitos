---
created: 2026-05-14
---

# Extract arithmetic parser from variables_parser.rs

The shared arithmetic body in \`parser/src/arithmetic.rs\` (extracted in commit \`0545776\`) is used by \`req\` and \`set\`, but \`evaluate_expression_internal\` in \`parser/src/parsers/variables_parser.rs\` (~lines 285-541) is still a third hand-rolled copy of the same recursive-descent grammar (\`parse_additive\` → \`parse_multiplicative\` → \`parse_unary\` → \`parse_primary\`), with its own Token enum, its own \`i64::MIN\` literal folding, and its own checked-arithmetic semantics.

Today this site constant-folds at parse time (no AST), but it shares the same overflow / div-by-zero / unary-minus edges as the shared body. Drift risk: a bug fix in the shared body (e.g. the recent depth cap from PR #75) does not propagate.

Surfaced by the round-6 \`/rust-review\` of PR #75 (logical-operators-in-req).

## Acceptance

- [ ] Port \`evaluate_expression_internal\` to drive \`ArithmeticSource\` and \`parse_arithmetic_expression\`.
- [ ] Add a small evaluation wrapper for the constant-folding case.
- [ ] Confirm unit tests still pass and that \`int x = -9223372036854775808\` produces the same diagnostic as the equivalent \`set x = -9223372036854775808\` and \`req x > -9223372036854775808\` shapes.
- [ ] All gates green: \`cargo fmt --check\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo test\`, \`./bin/run-compat\`.
