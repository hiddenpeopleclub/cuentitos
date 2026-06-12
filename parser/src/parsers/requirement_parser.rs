//! Parser for `req <boolean-expression>` statements.
//!
//! The boolean grammar (lowercase `and`/`or`/`not`, parens, and comparison
//! leaves) lives in [`crate::boolean_expression`]; this module is a thin
//! wrapper that:
//!
//! 1. Strips the leading `req` keyword.
//! 2. Delegates to [`parse_boolean_expression`] to build the tree.
//! 3. Walks each leaf comparison through [`infer_type`] so type/ordering
//!    errors are surfaced with their original Rust diagnostics.
//! 4. Maps the parser's typed errors into [`RequirementParseError`]
//!    variants ready for [`crate::ParseError`] formatting.

use cuentitos_common::{
    BooleanExpression, ComparisonOperator, Database, Expression, RequirementStatement, ValueKind,
    VariableId,
};

use crate::boolean_expression::{
    parse_boolean_expression, BooleanParseError, LogicalKeyword, VariableResolver,
};
use crate::parsers::type_inference::{infer_type, TypeInferenceError};

/// Result of parsing a `req` line.
///
/// Not `Eq`: the boolean expression may carry `Value::Float` literals (`f64`
/// has no total equality).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRequirement {
    pub expression: BooleanExpression,
}

/// Errors specific to parsing a `req` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementParseError {
    /// `req` followed by no condition.
    MissingCondition,
    /// LHS or RHS referenced a variable that was never declared.
    UndefinedVariable { name: String },
    /// Generic structural failure that surfaces as
    /// `Malformed expression in 'req': '<source>'`.
    MalformedExpression { expression: String },
    /// The tokenizer hit a symbol that isn't part of any `req`-grammar
    /// token (e.g. `&`, `|`, `~`). Not all of these are comparison
    /// operators — the diagnostic must say "unknown operator", not
    /// "unknown comparison operator".
    UnknownSymbol { symbol: String },
    /// LHS and RHS of a comparison inferred to different kinds.
    TypeMismatch { left: ValueKind, right: ValueKind },
    /// A comparison directly pairs two operands of different declared
    /// kinds (e.g. `door_open = 1`). Carries each operand's rendered
    /// source token and kind so the diagnostic can name both — e.g.
    /// `cannot compare bool 'door_open' with int '1'`.
    ComparisonTypeMismatch {
        left_kind: ValueKind,
        left_token: String,
        right_kind: ValueKind,
        right_token: String,
    },
    /// An ordering operator was applied to a non-ordered kind.
    NonOrderedComparison {
        operator: ComparisonOperator,
        kind: ValueKind,
    },
    /// An arithmetic subexpression operates on a non-numeric kind.
    NonNumericArithmetic { kind: ValueKind },

    // Logical-operator-specific errors with rich payloads.
    /// `and`/`or` had a bare arithmetic operand (left or right) instead of
    /// a comparison.
    LogicalBareIntegerOperand { operator: LogicalKeyword },
    /// `not` had a bare arithmetic operand instead of a comparison.
    LogicalBareIntegerOperandOfNot,
    /// `and`/`or` had no left operand. Source is the trimmed condition.
    LogicalMissingLeftOperand {
        operator: LogicalKeyword,
        source: String,
    },
    /// `and`/`or` had no right operand. Source is the trimmed condition.
    LogicalMissingRightOperand {
        operator: LogicalKeyword,
        source: String,
    },
    /// `not` had no operand. Source is the trimmed condition.
    LogicalMissingNotOperand { source: String },
    /// Unbalanced parens in the condition. Source is the trimmed condition.
    LogicalUnbalancedParentheses { source: String },
    /// A literal in the condition's arithmetic exceeded the integer
    /// range. Carries the offending literal text.
    LiteralOverflow { literal: String },
    /// The boolean condition nested deeper than the parser's recursion
    /// cap. See [`crate::boolean_expression::MAX_EXPRESSION_DEPTH`].
    ExpressionTooDeep,
    /// `==` was used in a comparison. Cuentitos uses `=` for equality;
    /// this variant carries no payload because the caller knows the
    /// context. See [`BooleanParseError::DoubleEquals`].
    DoubleEquals,
}

/// Try to parse `content` as a `req` statement.
///
/// `content` should already have indentation stripped, and the caller is
/// responsible for filtering with [`is_requirement_line`] first —
/// calling this on a non-`req` line is a contract violation. In debug
/// builds it panics; in release it surfaces as a `MissingCondition` so
/// the parser still makes progress.
///
/// `pub(crate)` so the predicate-then-parse contract is enforced by
/// crate-level visibility — external callers cannot bypass
/// `is_requirement_line` and stumble into the misleading
/// `MissingCondition` fallback.
pub(crate) fn parse_requirement(
    content: &str,
    database: &Database,
) -> Result<ParsedRequirement, RequirementParseError> {
    let rest = match strip_keyword(content, "req") {
        StripResult::Stripped(rest) => rest,
        StripResult::BareKeyword => return Err(RequirementParseError::MissingCondition),
        StripResult::NotKeyword => {
            debug_assert!(
                false,
                "parse_requirement called on non-req line — caller must filter with is_requirement_line: {content:?}"
            );
            return Err(RequirementParseError::MissingCondition);
        }
    };

    let payload = rest.trim();
    if payload.is_empty() {
        return Err(RequirementParseError::MissingCondition);
    }

    let resolver = DatabaseResolver { database };
    let expression = match parse_boolean_expression(payload, &resolver) {
        Ok(expression) => expression,
        Err(error) => return Err(map_boolean_error(error, payload)),
    };

    // Walk each leaf comparison so type/ordering errors surface with the
    // same diagnostics the single-comparison parser used to emit.
    validate_leaves(&expression, database)?;

    Ok(ParsedRequirement { expression })
}

fn map_boolean_error(error: BooleanParseError, source: &str) -> RequirementParseError {
    match error {
        BooleanParseError::BareIntegerOperandOfLogical { operator } => {
            RequirementParseError::LogicalBareIntegerOperand { operator }
        }
        BooleanParseError::BareIntegerOperandOfNot => {
            RequirementParseError::LogicalBareIntegerOperandOfNot
        }
        BooleanParseError::BareIntegerAtTop => RequirementParseError::MalformedExpression {
            expression: source.to_string(),
        },
        BooleanParseError::MissingLeftOperand { operator } => {
            RequirementParseError::LogicalMissingLeftOperand {
                operator,
                source: source.to_string(),
            }
        }
        BooleanParseError::MissingRightOperand { operator } => {
            RequirementParseError::LogicalMissingRightOperand {
                operator,
                source: source.to_string(),
            }
        }
        BooleanParseError::MissingNotOperand => RequirementParseError::LogicalMissingNotOperand {
            source: source.to_string(),
        },
        BooleanParseError::UnbalancedParentheses => {
            RequirementParseError::LogicalUnbalancedParentheses {
                source: source.to_string(),
            }
        }
        BooleanParseError::UndefinedVariable { name } => {
            RequirementParseError::UndefinedVariable { name }
        }
        BooleanParseError::UnknownSymbol { symbol } => {
            RequirementParseError::UnknownSymbol { symbol }
        }
        BooleanParseError::LiteralOverflow { literal } => {
            RequirementParseError::LiteralOverflow { literal }
        }
        BooleanParseError::ExpressionTooDeep => RequirementParseError::ExpressionTooDeep,
        BooleanParseError::DoubleEquals => RequirementParseError::DoubleEquals,
        BooleanParseError::Malformed => RequirementParseError::MalformedExpression {
            expression: source.to_string(),
        },
    }
}

// TODO: thread a sub-expression index or snippet through TypeMismatch /
// NonOrderedComparison / NonNumericArithmetic when non-Integer kinds
// land. Today the first failing comparison surfaces with no breadcrumb
// about which leaf in the tree it was — fine while Integer is the only
// kind, but `a > 0 and b > c` with a Float `c` will be hard to debug.
fn validate_leaves(
    expression: &BooleanExpression,
    database: &Database,
) -> Result<(), RequirementParseError> {
    match expression {
        BooleanExpression::Comparison(statement) => validate_comparison(statement, database),
        BooleanExpression::And(left, right) | BooleanExpression::Or(left, right) => {
            validate_leaves(left, database)?;
            validate_leaves(right, database)
        }
        BooleanExpression::Not(inner) => validate_leaves(inner, database),
    }
}

fn validate_comparison(
    statement: &RequirementStatement,
    database: &Database,
) -> Result<(), RequirementParseError> {
    let left_kind = infer_kind(&statement.left, database)?;
    let right_kind = infer_kind(&statement.right, database)?;
    if left_kind != right_kind {
        return Err(RequirementParseError::ComparisonTypeMismatch {
            left_kind,
            left_token: render_operand(&statement.left, database),
            right_kind,
            right_token: render_operand(&statement.right, database),
        });
    }
    if statement.operator.requires_ordering() && !left_kind.is_ordered() {
        return Err(RequirementParseError::NonOrderedComparison {
            operator: statement.operator,
            kind: left_kind,
        });
    }
    Ok(())
}

/// Render a comparison operand back to the source token a reader would
/// recognize: a variable's declared name, or a literal's textual value
/// (`1`, `true`). Compound arithmetic doesn't reduce to a single token, so
/// it falls back to a generic word — the bool comparison diagnostics that
/// consume this only ever pair a variable or literal across the operator.
fn render_operand(expression: &Expression, database: &Database) -> String {
    match expression {
        Expression::Variable(id) => database.variables[*id].name.clone(),
        Expression::Literal(value) => value.to_string(),
        Expression::Binary { .. } => "expression".to_string(),
    }
}

fn infer_kind(
    expression: &cuentitos_common::Expression,
    database: &Database,
) -> Result<ValueKind, RequirementParseError> {
    infer_type(expression, database).map_err(|err| match err {
        TypeInferenceError::Mismatch { left, right, .. } => {
            RequirementParseError::TypeMismatch { left, right }
        }
        TypeInferenceError::NonNumericArithmetic { kind, .. } => {
            RequirementParseError::NonNumericArithmetic { kind }
        }
    })
}

struct DatabaseResolver<'a> {
    database: &'a Database,
}

impl VariableResolver for DatabaseResolver<'_> {
    fn resolve(&self, name: &str) -> Option<VariableId> {
        self.database.variable_id(name)
    }

    fn kind_of(&self, id: VariableId) -> Option<ValueKind> {
        self.database
            .variables
            .get(id)
            .map(|variable| variable.kind())
    }

    fn enum_variants(&self, id: VariableId) -> Option<Vec<String>> {
        self.database
            .variables
            .get(id)
            .and_then(|variable| variable.default.enum_variants())
            .map(<[String]>::to_vec)
    }
}

/// Cheap predicate: does `content` (already trimmed of indentation)
/// begin with the `req` keyword followed by ASCII whitespace? Callers
/// must filter with this before [`parse_requirement`] — calling
/// `parse_requirement` on anything else is a contract violation.
pub fn is_requirement_line(content: &str) -> bool {
    matches!(
        strip_keyword(content, "req"),
        StripResult::Stripped(_) | StripResult::BareKeyword
    )
}

enum StripResult<'a> {
    Stripped(&'a str),
    BareKeyword,
    NotKeyword,
}

/// Same shape as `set_parser::strip_keyword`: tolerates space *or* tab
/// after the keyword so `req\tx > 5` parses cleanly.
fn strip_keyword<'a>(content: &'a str, keyword: &str) -> StripResult<'a> {
    if content == keyword {
        return StripResult::BareKeyword;
    }
    let Some(rest) = content.strip_prefix(keyword) else {
        return StripResult::NotKeyword;
    };
    let mut chars = rest.chars();
    match chars.next() {
        Some(c) if c.is_ascii_whitespace() => {
            StripResult::Stripped(rest[c.len_utf8()..].trim_start())
        }
        _ => StripResult::NotKeyword,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuentitos_common::{Expression, Value, Variable};

    fn db_with(vars: &[&str]) -> Database {
        let mut db = Database::new();
        for name in vars {
            db.add_variable(Variable::new_integer(*name, 0));
        }
        db
    }

    fn assert_comparison(
        expression: &BooleanExpression,
        expected_operator: ComparisonOperator,
    ) -> &RequirementStatement {
        match expression {
            BooleanExpression::Comparison(stmt) => {
                assert_eq!(stmt.operator, expected_operator);
                stmt
            }
            other => panic!("expected Comparison, got {:?}", other),
        }
    }

    #[test]
    fn parses_each_comparison_operator() {
        let db = db_with(&["x"]);
        for (input, expected_operator) in [
            ("req x > 0", ComparisonOperator::Greater),
            ("req x < 0", ComparisonOperator::Less),
            ("req x >= 0", ComparisonOperator::GreaterOrEqual),
            ("req x <= 0", ComparisonOperator::LessOrEqual),
            ("req x = 0", ComparisonOperator::Equal),
            ("req x != 0", ComparisonOperator::NotEqual),
        ] {
            let parsed = parse_requirement(input, &db).unwrap();
            let stmt = assert_comparison(&parsed.expression, expected_operator);
            assert_eq!(stmt.left, Expression::Variable(0));
            assert_eq!(stmt.right, Expression::Literal(Value::Integer(0)));
        }
    }

    #[test]
    fn parses_logical_and() {
        let db = db_with(&["x", "y"]);
        let parsed = parse_requirement("req x > 0 and y > 0", &db).unwrap();
        assert!(matches!(parsed.expression, BooleanExpression::And(_, _)));
    }

    #[test]
    fn parses_logical_or() {
        let db = db_with(&["x", "y"]);
        let parsed = parse_requirement("req x > 0 or y > 0", &db).unwrap();
        assert!(matches!(parsed.expression, BooleanExpression::Or(_, _)));
    }

    #[test]
    fn parses_logical_not() {
        let db = db_with(&["x"]);
        let parsed = parse_requirement("req not x > 0", &db).unwrap();
        assert!(matches!(parsed.expression, BooleanExpression::Not(_)));
    }

    #[test]
    fn parses_arithmetic_rhs() {
        let db = db_with(&["x", "y"]);
        let parsed = parse_requirement("req x > y + 1", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Greater);
        assert!(matches!(stmt.right, Expression::Binary { .. }));
    }

    #[test]
    fn parses_negative_literal_rhs() {
        let db = db_with(&["x"]);
        let parsed = parse_requirement("req x > -10", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Greater);
        assert_eq!(stmt.right, Expression::Literal(Value::Integer(-10)));
    }

    #[test]
    fn returns_undefined_for_lhs() {
        let db = db_with(&[]);
        assert_eq!(
            parse_requirement("req mana > 0", &db).unwrap_err(),
            RequirementParseError::UndefinedVariable {
                name: "mana".to_string()
            }
        );
    }

    #[test]
    fn returns_undefined_for_rhs_inside_expression() {
        let db = db_with(&["health"]);
        assert_eq!(
            parse_requirement("req health > 5 + mana", &db).unwrap_err(),
            RequirementParseError::UndefinedVariable {
                name: "mana".to_string()
            }
        );
    }

    #[test]
    fn returns_malformed_for_dangling_operator() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x > 5 +", &db).unwrap_err(),
            RequirementParseError::MalformedExpression {
                expression: "x > 5 +".to_string()
            }
        );
    }

    #[test]
    fn returns_unknown_symbol_for_tilde() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x ~ 5", &db).unwrap_err(),
            RequirementParseError::UnknownSymbol {
                symbol: "~".to_string()
            }
        );
    }

    #[test]
    fn returns_unknown_symbol_for_ampersand() {
        // `&` is the most common typo for `and` — surface it as an
        // unknown operator (not "unknown comparison operator", since `&`
        // isn't a comparison operator at all).
        let db = db_with(&["x", "y"]);
        assert_eq!(
            parse_requirement("req x > 0 & y > 0", &db).unwrap_err(),
            RequirementParseError::UnknownSymbol {
                symbol: "&".to_string()
            }
        );
    }

    #[test]
    fn is_requirement_line_filters_non_keyword_lines() {
        assert!(is_requirement_line("req x > 0"));
        assert!(is_requirement_line("req\tx > 0"));
        assert!(is_requirement_line("req"));
        assert!(!is_requirement_line("Hello world"));
        assert!(!is_requirement_line("require x > 0"));
    }

    #[test]
    fn bare_keyword_is_missing_condition() {
        let db = db_with(&[]);
        assert_eq!(
            parse_requirement("req", &db).unwrap_err(),
            RequirementParseError::MissingCondition
        );
    }

    #[test]
    fn tab_after_keyword_parses() {
        let db = db_with(&["x"]);
        let parsed = parse_requirement("req\tx > 0", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Greater);
        assert_eq!(stmt.left, Expression::Variable(0));
    }

    #[test]
    fn rejects_bare_integer_left_of_and() {
        let db = db_with(&["health", "shield"]);
        assert_eq!(
            parse_requirement("req health and shield > 0", &db).unwrap_err(),
            RequirementParseError::LogicalBareIntegerOperand {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_right_of_and() {
        let db = db_with(&["x", "y"]);
        assert_eq!(
            parse_requirement("req x > 0 and y", &db).unwrap_err(),
            RequirementParseError::LogicalBareIntegerOperand {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_right_of_or() {
        let db = db_with(&["x", "y"]);
        assert_eq!(
            parse_requirement("req x > 0 or y", &db).unwrap_err(),
            RequirementParseError::LogicalBareIntegerOperand {
                operator: LogicalKeyword::Or,
            }
        );
    }

    #[test]
    fn rejects_missing_right_operand_and() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x > 0 and", &db).unwrap_err(),
            RequirementParseError::LogicalMissingRightOperand {
                operator: LogicalKeyword::And,
                source: "x > 0 and".to_string(),
            }
        );
    }

    #[test]
    fn rejects_unbalanced_open_paren() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req (x > 0 and x < 10", &db).unwrap_err(),
            RequirementParseError::LogicalUnbalancedParentheses {
                source: "(x > 0 and x < 10".to_string(),
            }
        );
    }

    #[test]
    fn rejects_literal_overflow_with_literal_in_error() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x > 99999999999999999999", &db).unwrap_err(),
            RequirementParseError::LiteralOverflow {
                literal: "99999999999999999999".to_string(),
            }
        );
    }

    #[test]
    fn rejects_double_equals_with_dedicated_error() {
        // `==` is what C/Python users type instead of `=` for equality.
        // The dedicated error lets the caller emit a message that names
        // the right operator instead of falling through to "malformed".
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x == 5", &db).unwrap_err(),
            RequirementParseError::DoubleEquals
        );
    }

    // -- bool `req` ------------------------------------------------------

    fn db_with_bools(vars: &[&str]) -> Database {
        let mut db = Database::new();
        for name in vars {
            db.add_variable(Variable::new_boolean(*name, false));
        }
        db
    }

    #[test]
    fn truthiness_shortcut_desugars_to_equals_true() {
        // `req flag` on a bool variable is the truthiness shortcut: it
        // desugars to `flag = true`.
        let db = db_with_bools(&["flag"]);
        let parsed = parse_requirement("req flag", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Equal);
        assert_eq!(stmt.left, Expression::Variable(0));
        assert_eq!(stmt.right, Expression::Literal(Value::Boolean(true)));
    }

    #[test]
    fn truthiness_shortcut_negated_with_not() {
        let db = db_with_bools(&["flag"]);
        let parsed = parse_requirement("req not flag", &db).unwrap();
        match parsed.expression {
            BooleanExpression::Not(inner) => {
                assert_comparison(&inner, ComparisonOperator::Equal);
            }
            other => panic!("expected Not, got {:?}", other),
        }
    }

    #[test]
    fn truthiness_shortcuts_combine_with_and() {
        let db = db_with_bools(&["a", "b"]);
        let parsed = parse_requirement("req a and b", &db).unwrap();
        assert!(matches!(parsed.expression, BooleanExpression::And(_, _)));
    }

    #[test]
    fn bare_int_operand_is_not_truthiness() {
        // The truthiness shortcut is bool-only; a bare int operand keeps
        // its existing malformed-expression diagnostic.
        let db = db_with(&["health"]);
        assert_eq!(
            parse_requirement("req health", &db).unwrap_err(),
            RequirementParseError::MalformedExpression {
                expression: "health".to_string(),
            }
        );
    }

    #[test]
    fn compares_bool_variable_to_literal() {
        let db = db_with_bools(&["flag"]);
        let parsed = parse_requirement("req flag = true", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Equal);
        assert_eq!(stmt.left, Expression::Variable(0));
        assert_eq!(stmt.right, Expression::Literal(Value::Boolean(true)));
    }

    #[test]
    fn compares_two_bool_variables() {
        let db = db_with_bools(&["a", "b"]);
        let parsed = parse_requirement("req a != b", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::NotEqual);
        assert_eq!(stmt.left, Expression::Variable(0));
        assert_eq!(stmt.right, Expression::Variable(1));
    }

    #[test]
    fn ordering_operator_on_bool_is_rejected() {
        let db = db_with_bools(&["flag"]);
        assert_eq!(
            parse_requirement("req flag > false", &db).unwrap_err(),
            RequirementParseError::NonOrderedComparison {
                operator: ComparisonOperator::Greater,
                kind: ValueKind::Boolean,
            }
        );
    }

    #[test]
    fn comparing_bool_to_int_literal_is_type_mismatch() {
        let db = db_with_bools(&["flag"]);
        assert_eq!(
            parse_requirement("req flag = 1", &db).unwrap_err(),
            RequirementParseError::ComparisonTypeMismatch {
                left_kind: ValueKind::Boolean,
                left_token: "flag".to_string(),
                right_kind: ValueKind::Integer,
                right_token: "1".to_string(),
            }
        );
    }

    #[test]
    fn comparing_bool_to_int_variable_is_type_mismatch() {
        let mut db = db_with_bools(&["flag"]);
        db.add_variable(Variable::new_integer("health", 0));
        assert_eq!(
            parse_requirement("req flag = health", &db).unwrap_err(),
            RequirementParseError::ComparisonTypeMismatch {
                left_kind: ValueKind::Boolean,
                left_token: "flag".to_string(),
                right_kind: ValueKind::Integer,
                right_token: "health".to_string(),
            }
        );
    }

    #[test]
    fn truthiness_shortcut_on_undeclared_is_undefined_variable() {
        let db = db_with_bools(&["flag"]);
        assert_eq!(
            parse_requirement("req missing", &db).unwrap_err(),
            RequirementParseError::UndefinedVariable {
                name: "missing".to_string(),
            }
        );
    }

    // -- string `req` ----------------------------------------------------

    fn db_with_strings(vars: &[(&str, &str)]) -> Database {
        let mut db = Database::new();
        for (name, default) in vars {
            db.add_variable(Variable::new(*name, Value::String((*default).to_string())));
        }
        db
    }

    #[test]
    fn parses_string_literal_rhs_for_equal_and_not_equal() {
        let db = db_with_strings(&[("name", "Aria")]);
        for (input, expected_operator, expected_literal) in [
            ("req name = \"Aria\"", ComparisonOperator::Equal, "Aria"),
            (
                "req name != \"Brenn\"",
                ComparisonOperator::NotEqual,
                "Brenn",
            ),
        ] {
            let parsed = parse_requirement(input, &db).unwrap();
            let stmt = assert_comparison(&parsed.expression, expected_operator);
            assert_eq!(stmt.left, Expression::Variable(0));
            assert_eq!(
                stmt.right,
                Expression::Literal(Value::String(expected_literal.into()))
            );
        }
    }

    #[test]
    fn string_literal_rhs_decodes_escapes() {
        let db = db_with_strings(&[("greeting", "hi\nthere")]);
        let parsed = parse_requirement("req greeting = \"hi\\nthere\"", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Equal);
        assert_eq!(
            stmt.right,
            Expression::Literal(Value::String("hi\nthere".into()))
        );
    }

    #[test]
    fn parses_string_variable_on_both_sides() {
        let db = db_with_strings(&[("a", "Aria"), ("b", "Aria")]);
        let parsed = parse_requirement("req a = b", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Equal);
        assert_eq!(stmt.left, Expression::Variable(0));
        assert_eq!(stmt.right, Expression::Variable(1));
    }

    #[test]
    fn comparing_string_to_int_literal_is_type_mismatch() {
        let db = db_with_strings(&[("name", "Aria")]);
        assert_eq!(
            parse_requirement("req name = 1", &db).unwrap_err(),
            RequirementParseError::ComparisonTypeMismatch {
                left_kind: ValueKind::String,
                left_token: "name".to_string(),
                right_kind: ValueKind::Integer,
                right_token: "1".to_string(),
            }
        );
    }

    #[test]
    fn ordering_operator_on_strings_is_rejected() {
        let db = db_with_strings(&[("name", "Aria")]);
        assert_eq!(
            parse_requirement("req name > \"Aaa\"", &db).unwrap_err(),
            RequirementParseError::NonOrderedComparison {
                operator: ComparisonOperator::Greater,
                kind: ValueKind::String,
            }
        );
    }

    #[test]
    fn bare_string_variable_has_no_truthiness_shortcut() {
        let db = db_with_strings(&[("name", "Aria")]);
        assert_eq!(
            parse_requirement("req name", &db).unwrap_err(),
            RequirementParseError::MalformedExpression {
                expression: "name".to_string(),
            }
        );
    }

    #[test]
    fn unterminated_string_literal_is_malformed() {
        let db = db_with_strings(&[("name", "Aria")]);
        assert_eq!(
            parse_requirement("req name = \"Aria", &db).unwrap_err(),
            RequirementParseError::MalformedExpression {
                expression: "name = \"Aria".to_string(),
            }
        );
    }

    // -- enum `req` ------------------------------------------------------

    fn db_with_enum(name: &str, variants: &[&str]) -> Database {
        let mut db = Database::new();
        db.add_variable(cuentitos_common::Variable::new(
            name,
            cuentitos_common::Value::EnumUnset {
                variants: variants.iter().map(|v| v.to_string()).collect(),
            },
        ));
        db
    }

    #[test]
    fn parses_variant_literal_rhs_for_equal_and_not_equal() {
        let db = db_with_enum("mood", &["happy", "sad"]);
        for (input, expected_operator, expected_variant) in [
            ("req mood = happy", ComparisonOperator::Equal, "happy"),
            ("req mood != sad", ComparisonOperator::NotEqual, "sad"),
        ] {
            let parsed = parse_requirement(input, &db).unwrap();
            let stmt = assert_comparison(&parsed.expression, expected_operator);
            assert_eq!(stmt.left, Expression::Variable(0));
            assert_eq!(
                stmt.right,
                Expression::Literal(cuentitos_common::Value::Enum {
                    variants: vec!["happy".to_string(), "sad".to_string()],
                    value: expected_variant.to_string(),
                })
            );
        }
    }

    #[test]
    fn variant_resolves_against_the_lhs_enum() {
        // Two enums share `happy`; `req weather = happy` resolves against
        // weather's own variant list, carrying weather's variants.
        let mut db = db_with_enum("mood", &["happy", "sad"]);
        db.add_variable(cuentitos_common::Variable::new(
            "weather",
            cuentitos_common::Value::EnumUnset {
                variants: vec!["sunny".to_string(), "happy".to_string()],
            },
        ));
        let parsed = parse_requirement("req weather = happy", &db).unwrap();
        let stmt = assert_comparison(&parsed.expression, ComparisonOperator::Equal);
        assert_eq!(stmt.left, Expression::Variable(1));
        assert_eq!(
            stmt.right,
            Expression::Literal(cuentitos_common::Value::Enum {
                variants: vec!["sunny".to_string(), "happy".to_string()],
                value: "happy".to_string(),
            })
        );
    }

    #[test]
    fn unknown_variant_on_rhs_of_enum_req_is_undefined() {
        // A bare identifier that is not a variant (and not a variable) falls
        // through to the arithmetic body, which reports it as undefined.
        let db = db_with_enum("mood", &["happy", "sad"]);
        assert_eq!(
            parse_requirement("req mood = ecstatic", &db).unwrap_err(),
            RequirementParseError::UndefinedVariable {
                name: "ecstatic".to_string(),
            }
        );
    }
}
