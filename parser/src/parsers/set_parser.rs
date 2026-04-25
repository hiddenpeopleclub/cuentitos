//! Parser for `set <var> <op> <expr>` statements.
//!
//! Returns a `ParsedSet` carrying the resolved variable id, the assignment
//! operator, and the parsed expression AST. Identifier resolution happens at
//! parse time; the expression is evaluated later by the runtime.

use cuentitos_common::{AssignOp, Database, Expr, VariableId, VariableKind};

use crate::expression::{parse_expression, ParseExprError, VariableResolver};
use crate::parsers::variables_parser::is_valid_identifier;

/// Result of parsing a `set` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSet {
    pub variable_id: VariableId,
    pub op: AssignOp,
    pub expression: Expr,
}

/// Errors specific to parsing a `set` statement.
///
/// `MalformedExpression` is returned for any syntactic problem in the RHS;
/// the original RHS string is preserved so the caller can format an error
/// message like `Malformed 'set' statement: '5 +'.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetParseError {
    /// The line did not begin with `set ` followed by something parseable.
    /// Carries the trimmed line so the caller can decide whether to emit a
    /// targeted error or fall through to other parsers.
    NotASetStatement,
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
    /// LHS or an RHS variable reference was declared with a non-integer
    /// kind. Today this is unreachable (only `VariableKind::Int` exists),
    /// but the check is wired so adding `Bool`/`Float`/`String` doesn't
    /// silently produce wrong arithmetic at runtime.
    NonIntegerVariable { name: String },
}

/// Try to parse `content` as a `set` statement.
///
/// `content` should already have indentation stripped. Returns
/// `Err(NotASetStatement)` if the line doesn't begin with the `set ` keyword
/// — callers fall through to other parsers in that case.
pub fn parse_set(content: &str, database: &Database) -> Result<ParsedSet, SetParseError> {
    let rest = match content.strip_prefix("set ") {
        Some(rest) => rest,
        None => {
            // Allow `set` as a bare keyword line to surface a targeted error
            // rather than being treated as a String. Other shapes fall
            // through.
            if content == "set" {
                return Err(SetParseError::MissingLhs);
            }
            return Err(SetParseError::NotASetStatement);
        }
    };

    let (lhs_raw, op, rhs_raw) = split_lhs_op_rhs(rest)?;
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

    if !matches!(database.variables[variable_id].kind, VariableKind::Int(_)) {
        return Err(SetParseError::NonIntegerVariable {
            name: lhs.to_string(),
        });
    }

    if rhs.is_empty() {
        return Err(SetParseError::MissingRhs);
    }

    let resolver = DatabaseResolver { database };
    let expression = match parse_expression(rhs, &resolver) {
        Ok(expr) => expr,
        Err(ParseExprError::UndefinedVariable { name }) => {
            return Err(SetParseError::UndefinedVariable { name });
        }
        Err(ParseExprError::Malformed) | Err(ParseExprError::Overflow) => {
            return Err(SetParseError::MalformedExpression {
                expression: rhs.to_string(),
            });
        }
    };

    check_int_only(&expression, database)?;

    Ok(ParsedSet {
        variable_id,
        op,
        expression,
    })
}

/// Walk `expr` and reject any `Expr::Var(id)` whose declared kind isn't
/// `Int`. Lets `apply_set` use `as_int().expect(...)` at runtime instead
/// of silently coercing future non-int variants to zero.
fn check_int_only(expr: &Expr, database: &Database) -> Result<(), SetParseError> {
    match expr {
        Expr::Lit(_) => Ok(()),
        Expr::Var(id) => {
            if matches!(database.variables[*id].kind, VariableKind::Int(_)) {
                Ok(())
            } else {
                Err(SetParseError::NonIntegerVariable {
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

/// Locate the assignment operator and split into `(lhs, op, rhs)`. Compound
/// operators (`+=`, `-=`, `*=`, `/=`) take precedence over plain `=`.
fn split_lhs_op_rhs(rest: &str) -> Result<(&str, AssignOp, &str), SetParseError> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'+' || c == b'-' || c == b'*' || c == b'/' {
            // Only treat as compound assignment if followed by `=`.
            if bytes.get(i + 1).copied() == Some(b'=') {
                let op = match c {
                    b'+' => AssignOp::AddAssign,
                    b'-' => AssignOp::SubAssign,
                    b'*' => AssignOp::MulAssign,
                    b'/' => AssignOp::DivAssign,
                    _ => unreachable!(),
                };
                let lhs = &rest[..i];
                let rhs = &rest[i + 2..];
                return Ok((lhs, op, rhs));
            }
            // A bare `+`/`-`/`*`/`/` in the LHS is invalid as an assignment;
            // no compound op found here, keep scanning for `=`.
            i += 1;
            continue;
        }
        if c == b'=' {
            let lhs = &rest[..i];
            let rhs = &rest[i + 1..];
            return Ok((lhs, AssignOp::Assign, rhs));
        }
        i += 1;
    }
    Err(SetParseError::MissingAssignment)
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
    fn parses_plain_assignment() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x = 5", &db).unwrap();
        assert_eq!(parsed.variable_id, 0);
        assert_eq!(parsed.op, AssignOp::Assign);
        assert_eq!(parsed.expression, Expr::Lit(5));
    }

    #[test]
    fn parses_compound_assignment() {
        let db = db_with(&["x"]);
        for (input, expected_op) in [
            ("set x += 1", AssignOp::AddAssign),
            ("set x -= 1", AssignOp::SubAssign),
            ("set x *= 1", AssignOp::MulAssign),
            ("set x /= 1", AssignOp::DivAssign),
        ] {
            let parsed = parse_set(input, &db).unwrap();
            assert_eq!(parsed.op, expected_op, "input: {}", input);
        }
    }

    #[test]
    fn parses_compound_with_no_whitespace() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x+=1", &db).unwrap();
        assert_eq!(parsed.op, AssignOp::AddAssign);
        assert_eq!(parsed.expression, Expr::Lit(1));
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
    fn returns_not_a_set_for_unrelated_lines() {
        let db = db_with(&[]);
        assert_eq!(
            parse_set("Hello", &db).unwrap_err(),
            SetParseError::NotASetStatement
        );
    }

    #[test]
    fn negative_literal_rhs_parses() {
        let db = db_with(&["x"]);
        let parsed = parse_set("set x = -50", &db).unwrap();
        assert_eq!(parsed.expression, Expr::Lit(-50));
    }
}
