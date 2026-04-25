//! Parser for `req <var> <op> <expr>` statements.
//!
//! Returns a `ParsedReq` carrying the resolved LHS variable id, the
//! comparison operator, and the parsed RHS expression AST. Identifier
//! resolution happens at parse time; the comparison is evaluated at runtime
//! by [`cuentitos_runtime`] to decide whether to skip the gated parent.

use cuentitos_common::{CompareOp, Database, Expr, VariableId, VariableKind};

use crate::expression::{parse_expression, ParseExprError, VariableResolver};
use crate::parsers::variables_parser::is_valid_identifier;

/// Result of parsing a `req` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedReq {
    pub variable_id: VariableId,
    pub op: CompareOp,
    pub expression: Expr,
}

/// Errors specific to parsing a `req` statement.
///
/// Mirrors the shape of [`super::set_parser::SetParseError`]: top-level
/// callers re-map these to `ParseError` variants with file/line context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReqParseError {
    /// The line did not begin with `req ` followed by something parseable.
    NotAReqStatement,
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
    /// LHS or an RHS variable reference was declared with a non-integer
    /// kind. Today this is unreachable (only `VariableKind::Int` exists).
    NonIntegerVariable { name: String },
}

/// Try to parse `content` as a `req` statement.
///
/// `content` should already have indentation stripped. Returns
/// `Err(NotAReqStatement)` if the line doesn't begin with the `req ` keyword.
pub fn parse_req(content: &str, database: &Database) -> Result<ParsedReq, ReqParseError> {
    let rest = match content.strip_prefix("req ") {
        Some(rest) => rest,
        None => {
            if content == "req" {
                return Err(ReqParseError::MissingLhs);
            }
            return Err(ReqParseError::NotAReqStatement);
        }
    };

    let payload = rest.trim();
    if payload.is_empty() {
        return Err(ReqParseError::MissingLhs);
    }

    // Split off the LHS: first whitespace-or-symbol-bounded identifier.
    let (lhs, after_lhs) = split_lhs(payload);
    if lhs.is_empty() {
        return Err(ReqParseError::MissingLhs);
    }
    if !is_valid_identifier(lhs) {
        return Err(ReqParseError::InvalidLhs {
            name: lhs.to_string(),
        });
    }

    let variable_id =
        database
            .variable_id(lhs)
            .ok_or_else(|| ReqParseError::UndefinedVariable {
                name: lhs.to_string(),
            })?;

    if !matches!(database.variables[variable_id].kind, VariableKind::Int(_)) {
        return Err(ReqParseError::NonIntegerVariable {
            name: lhs.to_string(),
        });
    }

    // Identify the comparison operator. Longer prefixes (`>=`, `<=`, `!=`)
    // must be tried before their single-char counterparts.
    let after_lhs = after_lhs.trim_start();
    let (op, after_op) = match parse_compare_op(after_lhs) {
        Some((op, rest)) => (op, rest),
        None => {
            if let Some(token) = leading_symbol_token(after_lhs) {
                return Err(ReqParseError::UnknownOperator { op: token });
            }
            return Err(ReqParseError::MalformedExpression {
                expression: payload.to_string(),
            });
        }
    };

    let rhs = after_op.trim();
    if rhs.is_empty() {
        return Err(ReqParseError::MalformedExpression {
            expression: payload.to_string(),
        });
    }

    let resolver = DatabaseResolver { database };
    let expression = match parse_expression(rhs, &resolver) {
        Ok(expr) => expr,
        Err(ParseExprError::UndefinedVariable { name }) => {
            return Err(ReqParseError::UndefinedVariable { name });
        }
        Err(ParseExprError::Malformed) | Err(ParseExprError::Overflow) => {
            return Err(ReqParseError::MalformedExpression {
                expression: payload.to_string(),
            });
        }
    };

    check_int_only(&expression, database)?;

    Ok(ParsedReq {
        variable_id,
        op,
        expression,
    })
}

/// Walk `expr` and reject any `Expr::Var(id)` whose declared kind isn't
/// `Int`. Mirrors `set_parser::check_int_only` so the runtime evaluator
/// can rely on every variable reference being an integer.
fn check_int_only(expr: &Expr, database: &Database) -> Result<(), ReqParseError> {
    match expr {
        Expr::Lit(_) => Ok(()),
        Expr::Var(id) => {
            if matches!(database.variables[*id].kind, VariableKind::Int(_)) {
                Ok(())
            } else {
                Err(ReqParseError::NonIntegerVariable {
                    name: database.variables[*id].name.clone(),
                })
            }
        }
        Expr::Binary { left, right, .. } => {
            check_int_only(left, database)?;
            check_int_only(right, database)
        }
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

/// Try to consume a comparison operator from the start of `s`. Returns
/// `(op, rest)` on success.
fn parse_compare_op(s: &str) -> Option<(CompareOp, &str)> {
    if let Some(rest) = s.strip_prefix(">=") {
        return Some((CompareOp::Ge, rest));
    }
    if let Some(rest) = s.strip_prefix("<=") {
        return Some((CompareOp::Le, rest));
    }
    if let Some(rest) = s.strip_prefix("!=") {
        return Some((CompareOp::Ne, rest));
    }
    if let Some(rest) = s.strip_prefix('=') {
        return Some((CompareOp::Eq, rest));
    }
    if let Some(rest) = s.strip_prefix('>') {
        return Some((CompareOp::Gt, rest));
    }
    if let Some(rest) = s.strip_prefix('<') {
        return Some((CompareOp::Lt, rest));
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
    use cuentitos_common::Variable;

    fn db_with(vars: &[&str]) -> Database {
        let mut db = Database::new();
        for name in vars {
            db.add_variable(Variable::new_int(*name, 0));
        }
        db
    }

    #[test]
    fn parses_each_comparison_operator() {
        let db = db_with(&["x"]);
        for (input, expected_op) in [
            ("req x > 0", CompareOp::Gt),
            ("req x < 0", CompareOp::Lt),
            ("req x >= 0", CompareOp::Ge),
            ("req x <= 0", CompareOp::Le),
            ("req x = 0", CompareOp::Eq),
            ("req x != 0", CompareOp::Ne),
        ] {
            let parsed = parse_req(input, &db).unwrap();
            assert_eq!(parsed.op, expected_op, "input: {}", input);
            assert_eq!(parsed.variable_id, 0);
            assert_eq!(parsed.expression, Expr::Lit(0));
        }
    }

    #[test]
    fn parses_arithmetic_rhs() {
        let db = db_with(&["x", "y"]);
        let parsed = parse_req("req x > y + 1", &db).unwrap();
        assert_eq!(parsed.op, CompareOp::Gt);
        assert!(matches!(parsed.expression, Expr::Binary { .. }));
    }

    #[test]
    fn parses_negative_literal_rhs() {
        let db = db_with(&["x"]);
        let parsed = parse_req("req x > -10", &db).unwrap();
        assert_eq!(parsed.expression, Expr::Lit(-10));
    }

    #[test]
    fn returns_undefined_for_lhs() {
        let db = db_with(&[]);
        assert_eq!(
            parse_req("req mana > 0", &db).unwrap_err(),
            ReqParseError::UndefinedVariable {
                name: "mana".to_string()
            }
        );
    }

    #[test]
    fn returns_undefined_for_rhs() {
        let db = db_with(&["health"]);
        assert_eq!(
            parse_req("req health > mana", &db).unwrap_err(),
            ReqParseError::UndefinedVariable {
                name: "mana".to_string()
            }
        );
    }

    #[test]
    fn returns_undefined_for_rhs_inside_expression() {
        let db = db_with(&["health"]);
        assert_eq!(
            parse_req("req health > 5 + mana", &db).unwrap_err(),
            ReqParseError::UndefinedVariable {
                name: "mana".to_string()
            }
        );
    }

    #[test]
    fn returns_malformed_for_dangling_operator() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_req("req x > 5 +", &db).unwrap_err(),
            ReqParseError::MalformedExpression {
                expression: "x > 5 +".to_string()
            }
        );
    }

    #[test]
    fn returns_unknown_operator_for_tilde() {
        let db = db_with(&["x"]);
        assert_eq!(
            parse_req("req x ~ 5", &db).unwrap_err(),
            ReqParseError::UnknownOperator {
                op: "~".to_string()
            }
        );
    }

    #[test]
    fn returns_not_a_req_for_unrelated_lines() {
        let db = db_with(&[]);
        assert_eq!(
            parse_req("Hello world", &db).unwrap_err(),
            ReqParseError::NotAReqStatement
        );
    }

    #[test]
    fn bare_keyword_is_missing_lhs() {
        let db = db_with(&[]);
        assert_eq!(
            parse_req("req", &db).unwrap_err(),
            ReqParseError::MissingLhs
        );
    }

    #[test]
    fn evaluates_compare_op() {
        assert!(CompareOp::Gt.apply(2, 1));
        assert!(!CompareOp::Gt.apply(1, 1));
        assert!(CompareOp::Ge.apply(1, 1));
        assert!(CompareOp::Le.apply(1, 1));
        assert!(CompareOp::Lt.apply(0, 1));
        assert!(CompareOp::Eq.apply(5, 5));
        assert!(!CompareOp::Eq.apply(5, 4));
        assert!(CompareOp::Ne.apply(5, 4));
        assert!(!CompareOp::Ne.apply(5, 5));
    }
}
