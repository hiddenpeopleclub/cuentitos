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
    BooleanExpression, ComparisonOperator, Database, RequirementStatement, ValueKind, VariableId,
};

use crate::boolean_expression::{
    parse_boolean_expression, BooleanParseError, LogicalKeyword, VariableResolver,
};
use crate::parsers::type_inference::{infer_type, TypeInferenceError};

/// Result of parsing a `req` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequirement {
    pub expression: BooleanExpression,
}

/// Errors specific to parsing a `req` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementParseError {
    /// Line did not begin with `req`.
    NotARequirementStatement,
    /// `req` followed by no condition.
    MissingCondition,
    /// LHS or RHS referenced a variable that was never declared.
    UndefinedVariable { name: String },
    /// Generic structural failure that surfaces as
    /// `Malformed expression in 'req': '<source>'`.
    MalformedExpression { expression: String },
    /// A symbol was found between operands that isn't one of the
    /// supported comparison operators. Carries the offending symbol.
    UnknownOperator { symbol: String },
    /// LHS and RHS of a comparison inferred to different kinds.
    TypeMismatch { left: ValueKind, right: ValueKind },
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
}

/// Try to parse `content` as a `req` statement.
///
/// `content` should already have indentation stripped. Returns
/// `Err(NotARequirementStatement)` if the line doesn't begin with the
/// `req` keyword.
pub fn parse_requirement(
    content: &str,
    database: &Database,
) -> Result<ParsedRequirement, RequirementParseError> {
    let rest = match strip_keyword(content, "req") {
        StripResult::Stripped(rest) => rest,
        StripResult::BareKeyword => return Err(RequirementParseError::MissingCondition),
        StripResult::NotKeyword => return Err(RequirementParseError::NotARequirementStatement),
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
        BooleanParseError::UnknownOperator { symbol } => {
            RequirementParseError::UnknownOperator { symbol }
        }
        BooleanParseError::LiteralOverflow { literal } => {
            RequirementParseError::LiteralOverflow { literal }
        }
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
        return Err(RequirementParseError::TypeMismatch {
            left: left_kind,
            right: right_kind,
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
    fn returns_unknown_operator_for_tilde() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_requirement("req x ~ 5", &db).unwrap_err(),
            RequirementParseError::UnknownOperator {
                symbol: "~".to_string()
            }
        );
    }

    #[test]
    fn returns_not_a_req_for_unrelated_lines() {
        let db = db_with(&[]);
        assert_eq!(
            parse_requirement("Hello world", &db).unwrap_err(),
            RequirementParseError::NotARequirementStatement
        );
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
}
