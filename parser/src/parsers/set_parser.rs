//! Parser for `set <var> <op> <expr>` statements.
//!
//! Returns a `ParsedSet` carrying the resolved variable id, the assignment
//! operator, and the parsed expression AST. Identifier resolution and type
//! inference happen at parse time; the expression is evaluated later by the
//! runtime.

use cuentitos_common::{AssignmentOperator, Database, Expression, ValueKind, VariableId};

use crate::expression::{parse_expression, ParseExpressionError, VariableResolver};
use crate::parsers::type_inference::{infer_type, TypeInferenceError};
use crate::parsers::variables_parser::is_valid_identifier;

/// Result of parsing a `set` line.
///
/// Not `Eq`: `expression` may carry a `Value::Float` literal (`f64` has no
/// total equality).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSet {
    pub variable_id: VariableId,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}

/// Errors specific to parsing a `set` statement.
///
/// `MalformedExpression` is returned for any syntactic problem in the RHS;
/// the original RHS string is preserved so the caller can format an error
/// message like `Malformed 'set' statement: '5 +'.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetParseError {
    /// LHS or RHS referenced a variable that was never declared.
    UndefinedVariable { name: String },
    /// RHS expression had a syntactic problem.
    MalformedExpression { expression: String },
    /// LHS variable name was syntactically invalid (didn't match identifier rules).
    InvalidLhs { name: String },
    /// LHS was missing entirely (e.g. `set = 5`).
    MissingLhs,
    /// No `=`-bearing operator was found.
    MissingAssignment,
    /// RHS was empty (e.g. `set x = `).
    MissingRhs,
    /// RHS expression's inferred kind doesn't match the LHS variable's kind.
    TypeMismatch {
        variable: String,
        expected: ValueKind,
        found: ValueKind,
    },
    /// A compound assignment (`+=` etc.) targets a non-numeric variable.
    NonNumericAssignment { variable: String, kind: ValueKind },
    /// A literal in the RHS arithmetic exceeded the integer range.
    /// Carries the offending literal text. Mirrors
    /// [`crate::parsers::requirement_parser::RequirementParseError::LiteralOverflow`]
    /// so the two sibling parsers produce parallel diagnostics.
    LiteralOverflow { literal: String },
}

/// Try to parse `content` as a `set` statement.
///
/// `content` should already have indentation stripped, and the caller is
/// responsible for filtering with [`is_set_line`] first — calling this
/// on a non-`set` line is a contract violation. In debug builds it
/// panics; in release it surfaces as a `MissingLhs` so the parser still
/// makes progress.
///
/// `pub(crate)` so the predicate-then-parse contract is enforced by
/// crate-level visibility — external callers cannot bypass `is_set_line`
/// and stumble into the misleading `MissingLhs` fallback.
pub(crate) fn parse_set(content: &str, database: &Database) -> Result<ParsedSet, SetParseError> {
    let rest = match strip_keyword(content, "set") {
        StripResult::Stripped(rest) => rest,
        StripResult::BareKeyword => return Err(SetParseError::MissingLhs),
        StripResult::NotKeyword => {
            debug_assert!(
                false,
                "parse_set called on non-set line — caller must filter with is_set_line: {content:?}"
            );
            return Err(SetParseError::MissingLhs);
        }
    };

    let (lhs_raw, operator, rhs_raw) = split_lhs_op_rhs(rest)?;
    let lhs = lhs_raw.trim();
    let rhs = rhs_raw.trim();

    if lhs.is_empty() {
        return Err(SetParseError::MissingLhs);
    }
    if !is_valid_identifier(lhs) {
        return Err(SetParseError::InvalidLhs {
            name: lhs.to_string(),
        });
    }

    let variable_id =
        database
            .variable_id(lhs)
            .ok_or_else(|| SetParseError::UndefinedVariable {
                name: lhs.to_string(),
            })?;

    if rhs.is_empty() {
        return Err(SetParseError::MissingRhs);
    }

    let resolver = DatabaseResolver { database };
    let expression = match parse_expression(rhs, &resolver) {
        Ok(expression) => expression,
        Err(ParseExpressionError::UndefinedVariable { name }) => {
            return Err(SetParseError::UndefinedVariable { name });
        }
        Err(ParseExpressionError::Overflow { literal }) => {
            return Err(SetParseError::LiteralOverflow { literal });
        }
        Err(ParseExpressionError::Malformed) => {
            return Err(SetParseError::MalformedExpression {
                expression: rhs.to_string(),
            });
        }
    };

    let lhs_kind = database.variables[variable_id].kind();
    let rhs_kind = match infer_type(&expression, database) {
        Ok(kind) => kind,
        Err(TypeInferenceError::Mismatch { left, right, .. }) => {
            return Err(SetParseError::TypeMismatch {
                variable: lhs.to_string(),
                expected: left,
                found: right,
            });
        }
        Err(TypeInferenceError::NonNumericArithmetic { kind, .. }) => {
            return Err(SetParseError::NonNumericAssignment {
                variable: lhs.to_string(),
                kind,
            });
        }
    };

    if lhs_kind != rhs_kind {
        return Err(SetParseError::TypeMismatch {
            variable: lhs.to_string(),
            expected: lhs_kind,
            found: rhs_kind,
        });
    }

    if operator.is_compound() && !lhs_kind.is_numeric() {
        return Err(SetParseError::NonNumericAssignment {
            variable: lhs.to_string(),
            kind: lhs_kind,
        });
    }

    Ok(ParsedSet {
        variable_id,
        operator,
        expression,
    })
}

/// Cheap predicate: does `content` (already trimmed of indentation) begin
/// with the `set` keyword followed by ASCII whitespace? Callers must
/// filter with this before [`parse_set`] — calling `parse_set` on
/// anything else is a contract violation.
pub fn is_set_line(content: &str) -> bool {
    matches!(
        strip_keyword(content, "set"),
        StripResult::Stripped(_) | StripResult::BareKeyword
    )
}

enum StripResult<'a> {
    Stripped(&'a str),
    BareKeyword,
    NotKeyword,
}

/// Strip a leading keyword followed by ASCII whitespace. Tolerates either a
/// space or a tab between the keyword and the rest of the content so that
/// `set\tx = 5` doesn't silently fall through to the String parser. Returns
/// `BareKeyword` when the line is exactly the keyword (caller surfaces a
/// targeted "missing LHS" error).
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

struct DatabaseResolver<'a> {
    database: &'a Database,
}

impl VariableResolver for DatabaseResolver<'_> {
    fn resolve(&self, name: &str) -> Option<VariableId> {
        self.database.variable_id(name)
    }
}

/// Locate the assignment operator and split into `(lhs, op, rhs)`. Compound
/// operators (`+=`, `-=`, `*=`, `/=`) take precedence over plain `=`.
fn split_lhs_op_rhs(rest: &str) -> Result<(&str, AssignmentOperator, &str), SetParseError> {
    // Safe to byte-index: is_valid_identifier rejects non-ASCII LHS,
    // and the operator characters we look for are all single-byte ASCII.
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'+' || c == b'-' || c == b'*' || c == b'/' {
            // Only treat as compound assignment if followed by `=`.
            if bytes.get(i + 1).copied() == Some(b'=') {
                let operator = match c {
                    b'+' => AssignmentOperator::AddAssign,
                    b'-' => AssignmentOperator::SubtractAssign,
                    b'*' => AssignmentOperator::MultiplyAssign,
                    b'/' => AssignmentOperator::DivideAssign,
                    _ => unreachable!(),
                };
                let lhs = &rest[..i];
                let rhs = &rest[i + 2..];
                return Ok((lhs, operator, rhs));
            }
            // A bare `+`/`-`/`*`/`/` in the LHS is invalid as an assignment;
            // no compound op found here, keep scanning for `=`.
            i += 1;
            continue;
        }
        if c == b'=' {
            let lhs = &rest[..i];
            let rhs = &rest[i + 1..];
            return Ok((lhs, AssignmentOperator::Assign, rhs));
        }
        i += 1;
    }
    Err(SetParseError::MissingAssignment)
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

    #[test]
    fn parses_plain_assignment() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x = 5", &db).unwrap();
        assert_eq!(parsed.variable_id, 0);
        assert_eq!(parsed.operator, AssignmentOperator::Assign);
        assert_eq!(parsed.expression, Expression::Literal(Value::Integer(5)));
    }

    #[test]
    fn parses_compound_assignment() {
        let db = db_with(&["x"]);
        for (input, expected_operator) in [
            ("set x += 1", AssignmentOperator::AddAssign),
            ("set x -= 1", AssignmentOperator::SubtractAssign),
            ("set x *= 1", AssignmentOperator::MultiplyAssign),
            ("set x /= 1", AssignmentOperator::DivideAssign),
        ] {
            let parsed = parse_set(input, &db).unwrap();
            assert_eq!(parsed.operator, expected_operator, "input: {input}");
        }
    }

    #[test]
    fn parses_compound_with_no_whitespace() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x+=1", &db).unwrap();
        assert_eq!(parsed.operator, AssignmentOperator::AddAssign);
        assert_eq!(parsed.expression, Expression::Literal(Value::Integer(1)));
    }

    #[test]
    fn returns_undefined_for_lhs() {
        let db = db_with(&["other"]);
        let err = parse_set("set unknown = 1", &db).unwrap_err();
        assert_eq!(
            err,
            SetParseError::UndefinedVariable {
                name: "unknown".to_string()
            }
        );
    }

    #[test]
    fn returns_undefined_for_rhs_variable() {
        let db = db_with(&["score"]);
        let err = parse_set("set score = health + 1", &db).unwrap_err();
        assert_eq!(
            err,
            SetParseError::UndefinedVariable {
                name: "health".to_string()
            }
        );
    }

    #[test]
    fn returns_malformed_for_dangling_operator() {
        let db = db_with(&["x"]);
        let err = parse_set("set x = 5 +", &db).unwrap_err();
        assert_eq!(
            err,
            SetParseError::MalformedExpression {
                expression: "5 +".to_string()
            }
        );
    }

    #[test]
    fn returns_literal_overflow_for_positive_literal() {
        // A bare positive literal above the `u64` range surfaces with
        // its text intact so the diagnostic can name it. Previously
        // collapsed into MalformedExpression because the set tokenizer
        // discarded the offending text on `u64::from_str` failure.
        let db = db_with(&["x"]);
        assert_eq!(
            parse_set("set x = 99999999999999999999", &db).unwrap_err(),
            SetParseError::LiteralOverflow {
                literal: "99999999999999999999".to_string(),
            }
        );
    }

    #[test]
    fn returns_literal_overflow_for_negative_literal() {
        // `-9223372036854775809` = -(i64::MAX + 2). The magnitude fits
        // in u64 so the tokenizer accepts it, then `parse_unary` folds
        // the sign and `negate_u64_literal` catches the overflow.
        let db = db_with(&["x"]);
        assert_eq!(
            parse_set("set x = -9223372036854775809", &db).unwrap_err(),
            SetParseError::LiteralOverflow {
                literal: "-9223372036854775809".to_string(),
            }
        );
    }

    #[test]
    fn returns_literal_overflow_for_positive_one_above_i64_max() {
        // `9223372036854775808` = i64::MAX + 1. Fits in u64 but not i64.
        // Surfaces from the shared arith `parse_primary` int branch.
        let db = db_with(&["x"]);
        assert_eq!(
            parse_set("set x = 9223372036854775808", &db).unwrap_err(),
            SetParseError::LiteralOverflow {
                literal: "9223372036854775808".to_string(),
            }
        );
    }

    #[test]
    fn is_set_line_filters_non_keyword_lines() {
        assert!(is_set_line("set x = 5"));
        assert!(is_set_line("set\tx = 5"));
        assert!(is_set_line("set"));
        assert!(!is_set_line("Hello"));
        assert!(!is_set_line("seth = 1"));
    }

    #[test]
    fn negative_literal_rhs_parses() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x = -50", &db).unwrap();
        assert_eq!(parsed.expression, Expression::Literal(Value::Integer(-50)));
    }

    #[test]
    fn tab_after_keyword_parses() {
        // Regression: previously `set\tx = 5` slipped past the parser's
        // `looks_like_set_line` filter but failed `strip_prefix("set ")`
        // and triggered an `unreachable!()` panic. The keyword stripper
        // now accepts any ASCII whitespace.
        let db = db_with(&["x"]);
        let parsed = parse_set("set\tx = 5", &db).unwrap();
        assert_eq!(parsed.variable_id, 0);
        assert_eq!(parsed.expression, Expression::Literal(Value::Integer(5)));
    }
}
