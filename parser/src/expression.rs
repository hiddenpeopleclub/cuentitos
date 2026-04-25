//! Shared integer expression parser.
//!
//! Used by `set` (and later `req`) statements. Identifiers are resolved to
//! [`VariableId`]s at parse time via the supplied [`VariableResolver`]; the
//! resulting AST is then stored on the block and evaluated at runtime
//! (see [`cuentitos_common::evaluate`]).
//!
//! Variable defaults (in `--- variables` blocks) use a different evaluator
//! that constant-folds at parse time and never produces an AST — see
//! `parsers::variables_parser::evaluate_expression`.

use cuentitos_common::{BinOp, Expr, VariableId};

/// Errors produced while parsing or resolving an expression at parse time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseExprError {
    /// Tokenization failed or the grammar didn't accept the input.
    Malformed,
    /// Identifier was not found in the supplied resolver.
    UndefinedVariable { name: String },
    /// A constant subexpression overflowed at parse time. Triggered only by
    /// negating a literal magnitude greater than `i64::MIN`.
    Overflow,
}

/// Look up a declared variable by name.
pub trait VariableResolver {
    fn resolve(&self, name: &str) -> Option<VariableId>;
}

impl<F: Fn(&str) -> Option<VariableId>> VariableResolver for F {
    fn resolve(&self, name: &str) -> Option<VariableId> {
        self(name)
    }
}

/// Parse `input` and resolve every identifier through `resolver`.
pub fn parse_expression(
    input: &str,
    resolver: &dyn VariableResolver,
) -> Result<Expr, ParseExprError> {
    let tokens = tokenize(input).map_err(|_| ParseExprError::Malformed)?;
    if tokens.is_empty() {
        return Err(ParseExprError::Malformed);
    }
    let mut pos = 0;
    let expr = parse_additive(&tokens, &mut pos, resolver)?;
    if pos != tokens.len() {
        return Err(ParseExprError::Malformed);
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Integer literals are tokenized as `u64` so that the magnitude of `i64::MIN`
/// (`9_223_372_036_854_775_808`) is representable. `parse_unary` folds a
/// leading `-` into the literal so the full negative range is usable.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Int(u64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenizeError;

fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        match c {
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            c if c.is_ascii_digit() => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        buf.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n: u64 = buf.parse().map_err(|_| TokenizeError)?;
                tokens.push(Token::Int(n));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        buf.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(buf));
            }
            _ => return Err(TokenizeError),
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Recursive-descent parser
// ---------------------------------------------------------------------------

fn parse_additive(
    tokens: &[Token],
    pos: &mut usize,
    resolver: &dyn VariableResolver,
) -> Result<Expr, ParseExprError> {
    let mut left = parse_multiplicative(tokens, pos, resolver)?;
    loop {
        let op = match tokens.get(*pos) {
            Some(Token::Plus) => BinOp::Add,
            Some(Token::Minus) => BinOp::Sub,
            _ => break,
        };
        *pos += 1;
        let right = parse_multiplicative(tokens, pos, resolver)?;
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_multiplicative(
    tokens: &[Token],
    pos: &mut usize,
    resolver: &dyn VariableResolver,
) -> Result<Expr, ParseExprError> {
    let mut left = parse_unary(tokens, pos, resolver)?;
    loop {
        let op = match tokens.get(*pos) {
            Some(Token::Star) => BinOp::Mul,
            Some(Token::Slash) => BinOp::Div,
            _ => break,
        };
        *pos += 1;
        let right = parse_unary(tokens, pos, resolver)?;
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_unary(
    tokens: &[Token],
    pos: &mut usize,
    resolver: &dyn VariableResolver,
) -> Result<Expr, ParseExprError> {
    match tokens.get(*pos) {
        Some(Token::Minus) => {
            *pos += 1;
            // Fold `-` directly into a following literal so that `i64::MIN`
            // (whose magnitude doesn't fit in `i64`) is representable.
            if let Some(Token::Int(n)) = tokens.get(*pos) {
                *pos += 1;
                return negate_u64_literal(*n).map(Expr::Lit);
            }
            let inner = parse_unary(tokens, pos, resolver)?;
            // For non-literal unary minus, lower to `0 - inner` so that
            // overflow-checked subtraction at runtime catches the `-i64::MIN`
            // edge case.
            Ok(Expr::Binary {
                op: BinOp::Sub,
                left: Box::new(Expr::Lit(0)),
                right: Box::new(inner),
            })
        }
        Some(Token::Plus) => {
            *pos += 1;
            parse_unary(tokens, pos, resolver)
        }
        _ => parse_primary(tokens, pos, resolver),
    }
}

fn parse_primary(
    tokens: &[Token],
    pos: &mut usize,
    resolver: &dyn VariableResolver,
) -> Result<Expr, ParseExprError> {
    match tokens.get(*pos) {
        Some(Token::Int(n)) => {
            let n = *n;
            *pos += 1;
            if n > i64::MAX as u64 {
                return Err(ParseExprError::Overflow);
            }
            Ok(Expr::Lit(n as i64))
        }
        Some(Token::Ident(name)) => {
            let name = name.clone();
            *pos += 1;
            match resolver.resolve(&name) {
                Some(id) => Ok(Expr::Var(id)),
                None => Err(ParseExprError::UndefinedVariable { name }),
            }
        }
        Some(Token::LParen) => {
            *pos += 1;
            let expr = parse_additive(tokens, pos, resolver)?;
            match tokens.get(*pos) {
                Some(Token::RParen) => {
                    *pos += 1;
                    Ok(expr)
                }
                _ => Err(ParseExprError::Malformed),
            }
        }
        _ => Err(ParseExprError::Malformed),
    }
}

/// Compute `-(n as i64)` without intermediate overflow. Handles the unique
/// case of `i64::MIN`, whose absolute value is `(i64::MAX as u64) + 1`.
fn negate_u64_literal(n: u64) -> Result<i64, ParseExprError> {
    const ABS_MIN: u64 = (i64::MAX as u64) + 1;
    if n > ABS_MIN {
        return Err(ParseExprError::Overflow);
    }
    if n == ABS_MIN {
        return Ok(i64::MIN);
    }
    Ok(-(n as i64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuentitos_common::{evaluate, EvalExprError};
    use std::collections::HashMap;

    fn make_resolver(pairs: &[(&str, VariableId)]) -> impl VariableResolver {
        let map: HashMap<String, VariableId> =
            pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        move |name: &str| map.get(name).copied()
    }

    fn no_vars() -> impl VariableResolver {
        |_: &str| None
    }

    fn parse_and_eval(input: &str, vars: &[(&str, VariableId, i64)]) -> Result<i64, EvalExprError> {
        let resolver_pairs: Vec<(&str, VariableId)> =
            vars.iter().map(|(n, id, _)| (*n, *id)).collect();
        let resolver = make_resolver(&resolver_pairs);
        let expr = parse_expression(input, &resolver).expect("parse should succeed");
        let values: HashMap<VariableId, i64> = vars.iter().map(|(_, id, v)| (*id, *v)).collect();
        evaluate(&expr, &|id| values[&id])
    }

    #[test]
    fn parses_literal() {
        let resolver = no_vars();
        assert_eq!(parse_expression("42", &resolver).unwrap(), Expr::Lit(42));
    }

    #[test]
    fn parses_negative_literal() {
        let resolver = no_vars();
        assert_eq!(parse_expression("-5", &resolver).unwrap(), Expr::Lit(-5));
    }

    #[test]
    fn parses_i64_min_literal() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-9223372036854775808", &resolver).unwrap(),
            Expr::Lit(i64::MIN)
        );
    }

    #[test]
    fn parses_one_below_i64_min_overflows() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-9223372036854775809", &resolver).unwrap_err(),
            ParseExprError::Overflow
        );
    }

    #[test]
    fn parses_undefined_variable_errors() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("missing", &resolver).unwrap_err(),
            ParseExprError::UndefinedVariable {
                name: "missing".to_string()
            }
        );
    }

    #[test]
    fn parses_resolved_variable() {
        let resolver = make_resolver(&[("a", 7)]);
        assert_eq!(parse_expression("a", &resolver).unwrap(), Expr::Var(7));
    }

    #[test]
    fn parses_malformed_dangling_operator() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("5 +", &resolver).unwrap_err(),
            ParseExprError::Malformed
        );
    }

    #[test]
    fn parses_malformed_unbalanced_paren() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("(1 + 2", &resolver).unwrap_err(),
            ParseExprError::Malformed
        );
    }

    #[test]
    fn parses_empty_input_as_malformed() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("", &resolver).unwrap_err(),
            ParseExprError::Malformed
        );
        assert_eq!(
            parse_expression("   ", &resolver).unwrap_err(),
            ParseExprError::Malformed
        );
    }

    #[test]
    fn evaluates_arithmetic() {
        assert_eq!(parse_and_eval("1 + 2 * 3", &[]).unwrap(), 7);
        assert_eq!(parse_and_eval("(1 + 2) * 3", &[]).unwrap(), 9);
        assert_eq!(parse_and_eval("4 *5+ 6", &[]).unwrap(), 26);
    }

    #[test]
    fn evaluates_with_variables() {
        assert_eq!(
            parse_and_eval("a + b * c", &[("a", 0, 5), ("b", 1, 2), ("c", 2, 3)]).unwrap(),
            11
        );
    }

    #[test]
    fn evaluates_division_truncates_toward_zero() {
        assert_eq!(parse_and_eval("7 / 2", &[]).unwrap(), 3);
        assert_eq!(parse_and_eval("-7 / 2", &[]).unwrap(), -3);
        assert_eq!(parse_and_eval("7 / -2", &[]).unwrap(), -3);
        assert_eq!(parse_and_eval("-7 / -2", &[]).unwrap(), 3);
    }

    #[test]
    fn evaluates_division_by_zero() {
        assert_eq!(
            parse_and_eval("10 / 0", &[]).unwrap_err(),
            EvalExprError::DivisionByZero
        );
    }

    #[test]
    fn evaluates_division_by_zero_through_variable() {
        assert_eq!(
            parse_and_eval("10 / x", &[("x", 0, 0)]).unwrap_err(),
            EvalExprError::DivisionByZero
        );
    }

    #[test]
    fn evaluates_overflow_through_variable() {
        assert_eq!(
            parse_and_eval("big + 1", &[("big", 0, i64::MAX)]).unwrap_err(),
            EvalExprError::Overflow
        );
    }

    #[test]
    fn negation_of_variable_lowered_to_zero_minus() {
        let resolver = make_resolver(&[("x", 0)]);
        let expr = parse_expression("-x", &resolver).unwrap();
        assert_eq!(
            expr,
            Expr::Binary {
                op: BinOp::Sub,
                left: Box::new(Expr::Lit(0)),
                right: Box::new(Expr::Var(0)),
            }
        );
    }

    #[test]
    fn negation_of_i64_min_value_at_runtime_overflows() {
        let resolver = make_resolver(&[("x", 0)]);
        let expr = parse_expression("-x", &resolver).unwrap();
        let values = std::iter::once((0_usize, i64::MIN)).collect::<HashMap<_, _>>();
        assert_eq!(
            evaluate(&expr, &|id| values[&id]).unwrap_err(),
            EvalExprError::Overflow
        );
    }
}
