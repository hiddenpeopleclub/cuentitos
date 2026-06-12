//! Expression parser for single-expression contexts (`set` RHS).
//!
//! Tokenizes a UTF-8 string into the arithmetic-only token set, then
//! delegates to the shared body in [`crate::arithmetic`]. Identifiers
//! are resolved to [`VariableId`]s at parse time via the supplied
//! [`VariableResolver`]; the resulting AST is then stored on the block
//! and evaluated at runtime (see [`cuentitos_common::evaluate`]).
//!
//! Variable defaults (in `--- variables` blocks) use a different evaluator
//! that constant-folds at parse time and never produces an AST — see
//! `parsers::variables_parser::evaluate_expression`.
//!
//! The boolean-condition parser ([`crate::boolean_expression`]) uses the
//! same shared body to handle arithmetic operands of comparisons.

use cuentitos_common::{Expression, VariableId};

use crate::arithmetic::{
    parse_arithmetic_expression, ArithmeticError, ArithmeticSource, ArithmeticToken,
    ArithmeticTokenKind,
};

/// Errors produced while parsing or resolving an expression at parse time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseExpressionError {
    /// Tokenization failed or the grammar didn't accept the input.
    Malformed,
    /// Identifier was not found in the supplied resolver.
    UndefinedVariable { name: String },
    /// A literal exceeded the `i64` range. Carries the offending literal
    /// text (including any leading sign), parallel to
    /// [`crate::arithmetic::ArithmeticError::LiteralOverflow`], so the
    /// caller can format a diagnostic that names the literal.
    Overflow { literal: String },
    /// A float literal's magnitude exceeded the largest finite `f64`,
    /// parsing to ±infinity. Distinct from [`Overflow`](Self::Overflow) so
    /// the caller can surface a float-specific overflow message instead of
    /// the integer wording. Carries the offending literal text.
    FloatOverflow { literal: String },
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
) -> Result<Expression, ParseExpressionError> {
    let tokens = tokenize(input).map_err(|err| match err {
        TokenizeError::Malformed => ParseExpressionError::Malformed,
        TokenizeError::LiteralOverflow(literal) => ParseExpressionError::Overflow { literal },
        TokenizeError::FloatLiteralOverflow(literal) => {
            ParseExpressionError::FloatOverflow { literal }
        }
    })?;
    if tokens.is_empty() {
        return Err(ParseExpressionError::Malformed);
    }
    let mut source = SliceArithmeticSource {
        tokens: &tokens,
        position: 0,
        resolver,
        depth: 0,
    };
    let expression = parse_arithmetic_expression(&mut source).map_err(map_arithmetic_error)?;
    if source.position != tokens.len() {
        return Err(ParseExpressionError::Malformed);
    }
    Ok(expression)
}

fn map_arithmetic_error(error: ArithmeticError) -> ParseExpressionError {
    match error {
        ArithmeticError::Malformed | ArithmeticError::UnbalancedParentheses => {
            ParseExpressionError::Malformed
        }
        ArithmeticError::LiteralOverflow { literal } => ParseExpressionError::Overflow { literal },
        ArithmeticError::UndefinedVariable { name } => {
            ParseExpressionError::UndefinedVariable { name }
        }
        // The set-side parser doesn't currently surface depth-too-deep
        // with a dedicated diagnostic — fold into Malformed so the cap
        // still stops a stack-overflow attempt, even though the message
        // is generic. A follow-up can add a typed variant if the set
        // path ever sees depth-related authoring mistakes in the wild.
        ArithmeticError::ExpressionTooDeep => ParseExpressionError::Malformed,
    }
}

/// A [`ArithmeticSource`] backed by a pre-tokenized slice and a cursor.
struct SliceArithmeticSource<'a> {
    tokens: &'a [ArithmeticToken],
    position: usize,
    resolver: &'a dyn VariableResolver,
    /// Recursion depth accumulated by the shared arithmetic body via
    /// [`ArithmeticSource::enter_recursion`]. Mirrors the same cap as
    /// the boolean parser — same `MAX_EXPRESSION_DEPTH` constant — so
    /// `set x = ----…1` or `set x = (((…1)))` can't drive the parser
    /// stack to overflow.
    depth: usize,
}

impl<'a> ArithmeticSource for SliceArithmeticSource<'a> {
    fn peek_kind(&self) -> Option<ArithmeticTokenKind> {
        self.tokens.get(self.position).map(ArithmeticToken::kind)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn take_int(&mut self) -> Option<u64> {
        let ArithmeticToken::Int(n) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = *n;
        self.position += 1;
        Some(value)
    }

    fn take_bool(&mut self) -> Option<bool> {
        let ArithmeticToken::Bool(b) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = *b;
        self.position += 1;
        Some(value)
    }

    fn take_float(&mut self) -> Option<f64> {
        let ArithmeticToken::Float(value) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = *value;
        self.position += 1;
        Some(value)
    }

    // This source's tokenizer never emits a `Str` token — `set` lexes its
    // string RHS literals on a dedicated path before reaching the shared
    // arithmetic body — so `peek_kind` never yields `Str` and this is never
    // called. Implemented to satisfy the trait; always `None`.
    fn take_string(&mut self) -> Option<String> {
        let ArithmeticToken::Str(value) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = value.clone();
        self.position += 1;
        Some(value)
    }

    fn take_ident(&mut self) -> Option<String> {
        let ArithmeticToken::Ident(name) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = name.clone();
        self.position += 1;
        Some(value)
    }

    fn resolve(&self, name: &str) -> Option<VariableId> {
        self.resolver.resolve(name)
    }

    fn enter_recursion(&mut self) -> Result<(), ArithmeticError> {
        self.depth += 1;
        if self.depth > crate::boolean_expression::MAX_EXPRESSION_DEPTH {
            return Err(ArithmeticError::ExpressionTooDeep);
        }
        Ok(())
    }

    fn leave_recursion(&mut self) {
        self.depth -= 1;
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------
//
// Integer literals are tokenized as `u64` so that the magnitude of
// `i64::MIN` (`9_223_372_036_854_775_808`) is representable.
// `parse_unary` in the shared arithmetic body folds a leading `-` into the
// literal so the full negative range is usable.

/// Errors the set-side tokenizer can surface. Kept as an enum (rather
/// than a zero-sized struct) so a literal that exceeds the `u64` range
/// at lex time carries its source text — parallel to the boolean-side
/// tokenizer — and the eventual diagnostic can name the offending
/// literal instead of falling through to the generic "malformed" error.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenizeError {
    /// Catch-all for unrecognized symbols (anything that isn't a known
    /// operator, paren, identifier-start, or digit).
    Malformed,
    /// An integer literal exceeded `u64`. Carries the offending text so
    /// the caller can surface it as a literal-overflow diagnostic.
    LiteralOverflow(String),
    /// A `<digits>.<digits>` float literal whose magnitude exceeded the
    /// largest finite `f64`, parsing to ±infinity. Carries the offending
    /// text so the caller can surface a float-overflow diagnostic rather
    /// than silently storing the infinity.
    FloatLiteralOverflow(String),
}

fn tokenize(input: &str) -> Result<Vec<ArithmeticToken>, TokenizeError> {
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
                tokens.push(ArithmeticToken::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(ArithmeticToken::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(ArithmeticToken::Star);
            }
            '/' => {
                chars.next();
                tokens.push(ArithmeticToken::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(ArithmeticToken::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(ArithmeticToken::RParen);
            }
            c if c.is_ascii_digit() => {
                let mut buf = String::new();
                while let Some(&digit) = chars.peek() {
                    if digit.is_ascii_digit() {
                        buf.push(digit);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // A `.` immediately followed by at least one digit makes this
                // a float literal (`<digits>.<digits>`). A bare trailing dot
                // (`5.`) or a leading dot (`.5`) is not consumed here, so it
                // falls through as a stray symbol — matching the float-default
                // grammar, which accepts only `<digits>.<digits>`.
                let mut is_float = false;
                if chars.peek() == Some(&'.') {
                    let mut lookahead = chars.clone();
                    lookahead.next();
                    if matches!(lookahead.peek(), Some(d) if d.is_ascii_digit()) {
                        is_float = true;
                        buf.push('.');
                        chars.next();
                        while let Some(&digit) = chars.peek() {
                            if digit.is_ascii_digit() {
                                buf.push(digit);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
                if is_float {
                    // A `<digits>.<digits>` lexeme always parses; only an
                    // out-of-`f64`-range magnitude yields ±infinity, which
                    // `parse` reports as `Ok`, not `Err`. Treat any genuine
                    // parse failure as malformed, and a non-finite result as
                    // a float overflow — storing the infinity would silently
                    // accept a literal that the folded path (`finite_float`)
                    // rejects.
                    let value: f64 = buf.parse().map_err(|_| TokenizeError::Malformed)?;
                    if !value.is_finite() {
                        return Err(TokenizeError::FloatLiteralOverflow(buf.clone()));
                    }
                    tokens.push(ArithmeticToken::Float(value));
                } else {
                    // `u64::from_str` only fails here for magnitudes greater
                    // than `u64::MAX`. Preserve the literal text so the caller
                    // can surface the same overflow message as for in-range
                    // literals that exceed `i64`.
                    let parsed: u64 = buf
                        .parse()
                        .map_err(|_| TokenizeError::LiteralOverflow(buf.clone()))?;
                    tokens.push(ArithmeticToken::Int(parsed));
                }
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
                // `true`/`false` are boolean literals, not identifiers, so
                // `set flag = true` folds to a `Value::Boolean` rather than
                // resolving a (non-existent) variable named `true`.
                match buf.as_str() {
                    "true" => tokens.push(ArithmeticToken::Bool(true)),
                    "false" => tokens.push(ArithmeticToken::Bool(false)),
                    _ => tokens.push(ArithmeticToken::Ident(buf)),
                }
            }
            _ => return Err(TokenizeError::Malformed),
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuentitos_common::{evaluate, BinaryOperator, EvaluationError, Value};
    use std::collections::HashMap;

    fn make_resolver(pairs: &[(&str, VariableId)]) -> impl VariableResolver {
        let map: HashMap<String, VariableId> =
            pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        move |name: &str| map.get(name).copied()
    }

    fn no_vars() -> impl VariableResolver {
        |_: &str| None
    }

    fn parse_and_eval(
        input: &str,
        vars: &[(&str, VariableId, i64)],
    ) -> Result<i64, EvaluationError> {
        let resolver_pairs: Vec<(&str, VariableId)> =
            vars.iter().map(|(n, id, _)| (*n, *id)).collect();
        let resolver = make_resolver(&resolver_pairs);
        let expression = parse_expression(input, &resolver).expect("parse should succeed");
        let values: HashMap<VariableId, Value> = vars
            .iter()
            .map(|(_, id, v)| (*id, Value::Integer(*v)))
            .collect();
        match evaluate(&expression, &|id| &values[&id])?.into_owned() {
            Value::Integer(n) => Ok(n),
            Value::Boolean(_) | Value::Float(_) | Value::String(_) => {
                unreachable!("arithmetic folds only produce integers")
            }
        }
    }

    #[test]
    fn parses_literal() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("42", &resolver).unwrap(),
            Expression::Literal(Value::Integer(42))
        );
    }

    #[test]
    fn parses_true_and_false_as_bool_literals() {
        // `set flag = true` folds to a boolean literal, not an identifier
        // lookup of a (non-existent) variable named `true`.
        let resolver = no_vars();
        assert_eq!(
            parse_expression("true", &resolver).unwrap(),
            Expression::Literal(Value::Boolean(true))
        );
        assert_eq!(
            parse_expression("false", &resolver).unwrap(),
            Expression::Literal(Value::Boolean(false))
        );
    }

    #[test]
    fn parses_negative_literal() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-5", &resolver).unwrap(),
            Expression::Literal(Value::Integer(-5))
        );
    }

    #[test]
    fn parses_i64_min_literal() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-9223372036854775808", &resolver).unwrap(),
            Expression::Literal(Value::Integer(i64::MIN))
        );
    }

    #[test]
    fn parses_one_below_i64_min_overflows() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-9223372036854775809", &resolver).unwrap_err(),
            ParseExpressionError::Overflow {
                literal: "-9223372036854775809".to_string(),
            }
        );
    }

    #[test]
    fn parses_positive_overflow_carries_literal() {
        // A bare positive literal larger than i64::MAX surfaces with
        // its text intact so the caller can name it in the diagnostic.
        let resolver = no_vars();
        assert_eq!(
            parse_expression("99999999999999999999", &resolver).unwrap_err(),
            ParseExpressionError::Overflow {
                literal: "99999999999999999999".to_string(),
            }
        );
    }

    #[test]
    fn parses_undefined_variable_errors() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("missing", &resolver).unwrap_err(),
            ParseExpressionError::UndefinedVariable {
                name: "missing".to_string()
            }
        );
    }

    #[test]
    fn parses_resolved_variable() {
        let resolver = make_resolver(&[("a", 7)]);
        assert_eq!(
            parse_expression("a", &resolver).unwrap(),
            Expression::Variable(7)
        );
    }

    #[test]
    fn parses_malformed_dangling_operator() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("5 +", &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
    }

    #[test]
    fn parses_malformed_unbalanced_paren() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("(1 + 2", &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
    }

    #[test]
    fn rejects_deep_unary_minus_chain() {
        // 200 leading `-`s would stack-overflow the parser before the
        // arith side joined the depth cap. The set side surfaces
        // `ExpressionTooDeep` as `Malformed` for now — the test pins
        // that we *fail cleanly* rather than crashing.
        let resolver = no_vars();
        let mut input = String::new();
        for _ in 0..200 {
            input.push('-');
        }
        input.push('1');
        assert_eq!(
            parse_expression(&input, &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
    }

    #[test]
    fn rejects_deep_paren_nesting() {
        // 200 nested `(`s exercise the LParen recursion in the shared
        // arith body. Same fail-cleanly contract as the unary-minus
        // chain.
        let resolver = no_vars();
        let mut input = String::new();
        for _ in 0..200 {
            input.push('(');
        }
        input.push('1');
        for _ in 0..200 {
            input.push(')');
        }
        assert_eq!(
            parse_expression(&input, &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
    }

    #[test]
    fn parses_empty_input_as_malformed() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("", &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
        assert_eq!(
            parse_expression("   ", &resolver).unwrap_err(),
            ParseExpressionError::Malformed
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
            EvaluationError::DivisionByZero
        );
    }

    #[test]
    fn evaluates_division_by_zero_through_variable() {
        assert_eq!(
            parse_and_eval("10 / x", &[("x", 0, 0)]).unwrap_err(),
            EvaluationError::DivisionByZero
        );
    }

    #[test]
    fn evaluates_overflow_through_variable() {
        assert_eq!(
            parse_and_eval("big + 1", &[("big", 0, i64::MAX)]).unwrap_err(),
            EvaluationError::Overflow
        );
    }

    #[test]
    fn negation_of_variable_lowered_to_zero_minus() {
        let resolver = make_resolver(&[("x", 0)]);
        let expression = parse_expression("-x", &resolver).unwrap();
        assert_eq!(
            expression,
            Expression::Binary {
                operator: BinaryOperator::Subtract,
                left: Box::new(Expression::Literal(Value::Integer(0))),
                right: Box::new(Expression::Variable(0)),
            }
        );
    }

    #[test]
    fn negation_of_i64_min_value_at_runtime_overflows() {
        let resolver = make_resolver(&[("x", 0)]);
        let expression = parse_expression("-x", &resolver).unwrap();
        let values: HashMap<VariableId, Value> =
            std::iter::once((0_usize, Value::Integer(i64::MIN))).collect();
        assert_eq!(
            evaluate(&expression, &|id| &values[&id]).unwrap_err(),
            EvaluationError::Overflow
        );
    }

    fn eval_floatless(input: &str) -> Value {
        let resolver = no_vars();
        let expression = parse_expression(input, &resolver).expect("parse should succeed");
        evaluate(&expression, &|_| unreachable!("no variables"))
            .unwrap()
            .into_owned()
    }

    #[test]
    fn parses_float_literal() {
        let resolver = no_vars();
        assert_eq!(
            parse_expression("7.5", &resolver).unwrap(),
            Expression::Literal(Value::Float(7.5))
        );
    }

    #[test]
    fn parses_negative_float_literal() {
        // `-7.5` folds into a single negative literal, not `0.0 - 7.5`.
        let resolver = no_vars();
        assert_eq!(
            parse_expression("-7.5", &resolver).unwrap(),
            Expression::Literal(Value::Float(-7.5))
        );
    }

    #[test]
    fn bare_integer_stays_integer_literal() {
        // A digit run with no fractional part remains an `Int` token, so
        // integer `set`s are unaffected by float tokenization.
        let resolver = no_vars();
        assert_eq!(
            parse_expression("5", &resolver).unwrap(),
            Expression::Literal(Value::Integer(5))
        );
    }

    #[test]
    fn trailing_dot_is_not_a_float_literal() {
        // `5.` is not `<digits>.<digits>`: the `.` is left as a stray symbol
        // and the parse is malformed, matching the float-default grammar.
        let resolver = no_vars();
        assert_eq!(
            parse_expression("5.", &resolver).unwrap_err(),
            ParseExpressionError::Malformed
        );
    }

    #[test]
    fn evaluates_float_arithmetic_without_truncation() {
        assert_eq!(eval_floatless("7.0 / 2.0"), Value::Float(3.5));
        assert_eq!(eval_floatless("(5.0 + 3.0) * 2.0"), Value::Float(16.0));
    }

    #[test]
    fn float_division_by_zero_surfaces_at_eval() {
        let resolver = no_vars();
        let expression = parse_expression("10.0 / 0.0", &resolver).unwrap();
        assert_eq!(
            evaluate(&expression, &|_| unreachable!()).unwrap_err(),
            EvaluationError::DivisionByZero
        );
    }

    #[test]
    fn float_literal_beyond_f64_range_is_an_overflow_not_infinity() {
        // A lone `<digits>.<digits>` literal whose magnitude exceeds the
        // largest finite `f64` parses to infinity; reject it at tokenize time
        // rather than letting `Value::Float(inf)` reach a variable. `1e320`
        // (plain decimal, no exponent notation) is comfortably out of range.
        let resolver = no_vars();
        let literal = format!("1{}.0", "0".repeat(320));
        assert_eq!(
            parse_expression(&literal, &resolver).unwrap_err(),
            ParseExpressionError::FloatOverflow { literal }
        );
    }
}
