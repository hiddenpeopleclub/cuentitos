//! Parser for `req <var> <op> <expr>` statements.
//!
//! Returns a `ParsedRequirement` carrying the LHS as an [`Expression`], the
//! [`ComparisonOperator`], and the RHS [`Expression`]. The current grammar
//! restricts the LHS to a single variable identifier (wrapped as
//! [`Expression::Variable`]), but the data model is symmetric so future
//! grammar relaxation `req x + 1 > y * 2` is purely a parser change.

use cuentitos_common::{ComparisonOperator, Database, Expression, ValueKind, VariableId};

use crate::expression::{parse_expression, ParseExpressionError, VariableResolver};
use crate::parsers::type_inference::{infer_type, TypeInferenceError};
use crate::parsers::variables_parser::is_valid_identifier;

/// Result of parsing a `req` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRequirement {
    pub left: Expression,
    pub operator: ComparisonOperator,
    pub right: Expression,
}

/// Errors specific to parsing a `req` statement.
///
/// Top-level callers re-map these to `ParseError` variants with file/line
/// context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementParseError {
    /// The line did not begin with `req` followed by something parseable.
    NotARequirementStatement,
    /// LHS or RHS referenced a variable that was never declared.
    UndefinedVariable { name: String },
    /// RHS expression had a syntactic problem. Carries the full payload
    /// after `req ` so the caller can format an error message like
    /// `Malformed expression in 'req': 'x > 5 +'.`.
    MalformedExpression { expression: String },
    /// LHS variable name was syntactically invalid.
    InvalidLhs { name: String },
    /// LHS was missing entirely (e.g. `req`, `req > 5`).
    MissingLhs,
    /// A symbol was found between LHS and RHS that isn't one of the
    /// supported comparison operators. Carries the offending token.
    UnknownOperator { op: String },
    /// LHS and RHS inferred to different kinds.
    TypeMismatch { left: ValueKind, right: ValueKind },
    /// An ordering operator (`<`, `<=`, `>`, `>=`) was applied to a kind
    /// that doesn't support ordering. Today unreachable through normal
    /// parsing because only `Integer` exists and it is ordered; wired now
    /// so the error surface is ready for future kinds.
    NonOrderedComparison {
        operator: ComparisonOperator,
        kind: ValueKind,
    },
    /// An arithmetic subexpression operates on a non-numeric kind.
    NonNumericArithmetic { kind: ValueKind },
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
        StripResult::BareKeyword => return Err(RequirementParseError::MissingLhs),
        StripResult::NotKeyword => return Err(RequirementParseError::NotARequirementStatement),
    };

    let payload = rest.trim();
    if payload.is_empty() {
        return Err(RequirementParseError::MissingLhs);
    }

    let (lhs, after_lhs) = split_lhs(payload);
    if lhs.is_empty() {
        return Err(RequirementParseError::MissingLhs);
    }
    if !is_valid_identifier(lhs) {
        return Err(RequirementParseError::InvalidLhs {
            name: lhs.to_string(),
        });
    }

    let variable_id =
        database
            .variable_id(lhs)
            .ok_or_else(|| RequirementParseError::UndefinedVariable {
                name: lhs.to_string(),
            })?;

    let after_lhs = after_lhs.trim_start();
    let (operator, after_op) = match parse_compare_op(after_lhs) {
        Some((operator, rest)) => (operator, rest),
        None => {
            if let Some(token) = leading_symbol_token(after_lhs) {
                return Err(RequirementParseError::UnknownOperator { op: token });
            }
            return Err(RequirementParseError::MalformedExpression {
                expression: payload.to_string(),
            });
        }
    };

    let rhs = after_op.trim();
    if rhs.is_empty() {
        return Err(RequirementParseError::MalformedExpression {
            expression: payload.to_string(),
        });
    }

    let resolver = DatabaseResolver { database };
    let right = match parse_expression(rhs, &resolver) {
        Ok(expression) => expression,
        Err(ParseExpressionError::UndefinedVariable { name }) => {
            return Err(RequirementParseError::UndefinedVariable { name });
        }
        Err(ParseExpressionError::Malformed) | Err(ParseExpressionError::Overflow) => {
            return Err(RequirementParseError::MalformedExpression {
                expression: payload.to_string(),
            });
        }
    };

    let left = Expression::Variable(variable_id);

    let left_kind = infer_kind(&left, database)?;
    let right_kind = infer_kind(&right, database)?;

    if left_kind != right_kind {
        return Err(RequirementParseError::TypeMismatch {
            left: left_kind,
            right: right_kind,
        });
    }

    if operator.requires_ordering() && !left_kind.is_ordered() {
        return Err(RequirementParseError::NonOrderedComparison {
            operator,
            kind: left_kind,
        });
    }

    Ok(ParsedRequirement {
        left,
        operator,
        right,
    })
}

fn infer_kind(
    expression: &Expression,
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

/// See `set_parser::strip_keyword` — same shape: tolerates space *or* tab
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

/// Take the leading identifier-shaped run of characters from `s` and return
/// `(lhs, rest)`. An empty string is returned when `s` doesn't begin with an
/// identifier character — the caller surfaces the appropriate error.
fn split_lhs(s: &str) -> (&str, &str) {
    let end = s
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    s.split_at(end)
}

/// Try to consume a comparison operator from the start of `s`. Longer
/// prefixes (`>=`, `<=`, `!=`) must be tried before their single-char
/// counterparts.
fn parse_compare_op(s: &str) -> Option<(ComparisonOperator, &str)> {
    if let Some(rest) = s.strip_prefix(">=") {
        return Some((ComparisonOperator::GreaterOrEqual, rest));
    }
    if let Some(rest) = s.strip_prefix("<=") {
        return Some((ComparisonOperator::LessOrEqual, rest));
    }
    if let Some(rest) = s.strip_prefix("!=") {
        return Some((ComparisonOperator::NotEqual, rest));
    }
    if let Some(rest) = s.strip_prefix('=') {
        return Some((ComparisonOperator::Equal, rest));
    }
    if let Some(rest) = s.strip_prefix('>') {
        return Some((ComparisonOperator::Greater, rest));
    }
    if let Some(rest) = s.strip_prefix('<') {
        return Some((ComparisonOperator::Less, rest));
    }
    None
}

/// Read a contiguous run of operator-shaped characters from the start of
/// `s` to surface as an "Unknown comparison operator" payload. Returns
/// `None` when the leading char isn't operator-shaped (in which case the
/// caller falls back to a generic malformed-expression error).
fn leading_symbol_token(s: &str) -> Option<String> {
    let mut chars = s.chars();
    let first = chars.next()?;
    if first.is_ascii_alphanumeric() || first == '_' || first.is_whitespace() {
        return None;
    }
    let mut token = String::new();
    token.push(first);
    for c in chars {
        if c.is_ascii_alphanumeric() || c == '_' || c.is_whitespace() || c == '(' || c == ')' {
            break;
        }
        token.push(c);
    }
    Some(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuentitos_common::{Value, Variable};

    fn db_with(vars: &[&str]) -> Database {
        let mut db = Database::new();
        for name in vars {
            db.add_variable(Variable::new_integer(*name, 0));
        }
        db
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
            assert_eq!(parsed.operator, expected_operator, "input: {input}");
            assert_eq!(parsed.left, Expression::Variable(0));
            assert_eq!(parsed.right, Expression::Literal(Value::Integer(0)));
        }
    }

    #[test]
    fn parses_arithmetic_rhs() {
        let db = db_with(&["x", "y"]);
        let parsed = parse_requirement("req x > y + 1", &db).unwrap();
        assert_eq!(parsed.operator, ComparisonOperator::Greater);
        assert!(matches!(parsed.right, Expression::Binary { .. }));
    }

    #[test]
    fn parses_negative_literal_rhs() {
        let db = db_with(&["x"]);
        let parsed = parse_requirement("req x > -10", &db).unwrap();
        assert_eq!(parsed.right, Expression::Literal(Value::Integer(-10)));
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
    fn returns_undefined_for_rhs() {
        let db = db_with(&["health"]);
        assert_eq!(
            parse_requirement("req health > mana", &db).unwrap_err(),
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
                op: "~".to_string()
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
    fn bare_keyword_is_missing_lhs() {
        let db = db_with(&[]);
        assert_eq!(
            parse_requirement("req", &db).unwrap_err(),
            RequirementParseError::MissingLhs
        );
    }

    #[test]
    fn tab_after_keyword_parses() {
        // Regression: same shape as the set-parser tab fix.
        let db = db_with(&["x"]);
        let parsed = parse_requirement("req\tx > 0", &db).unwrap();
        assert_eq!(parsed.left, Expression::Variable(0));
        assert_eq!(parsed.operator, ComparisonOperator::Greater);
    }

    #[test]
    fn evaluates_compare_op_via_apply_integer() {
        assert!(ComparisonOperator::Greater.apply_integer(2, 1));
        assert!(!ComparisonOperator::Greater.apply_integer(1, 1));
        assert!(ComparisonOperator::GreaterOrEqual.apply_integer(1, 1));
        assert!(ComparisonOperator::LessOrEqual.apply_integer(1, 1));
        assert!(ComparisonOperator::Less.apply_integer(0, 1));
        assert!(ComparisonOperator::Equal.apply_integer(5, 5));
        assert!(!ComparisonOperator::Equal.apply_integer(5, 4));
        assert!(ComparisonOperator::NotEqual.apply_integer(5, 4));
        assert!(!ComparisonOperator::NotEqual.apply_integer(5, 5));
    }

    /// Once a non-`Integer` `ValueKind` exists, this test will be unblocked:
    /// build an `Expression::Literal(Value::<NewKind>(...))` and assign it
    /// to an integer variable; `parse_set` (here, `parse_requirement`'s
    /// sibling) should reject it with `TypeMismatch`. We cannot exercise
    /// the path today because there is only one variant, and faking a
    /// second variant just for the test would defeat the point.
    #[test]
    #[ignore = "unblocked by second ValueKind variant"]
    fn type_mismatch_is_reachable_for_future_kinds() {}
}
