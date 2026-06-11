//! Shared recursive-descent body for arithmetic expressions.
//!
//! Both the `set`/`req` single-expression parser ([`crate::expression`])
//! and the `req` boolean-expression parser ([`crate::boolean_expression`])
//! need to consume the same arithmetic sublanguage:
//!
//! ```text
//! additive       := multiplicative (`+`|`-` multiplicative)*
//! multiplicative := unary (`*`|`/` unary)*
//! unary          := `-` unary | `+` unary | primary
//! primary        := integer | identifier | `(` additive `)`
//! ```
//!
//! with the same operator precedence, the same `i64::MIN`-aware literal
//! negation, and the same `0 - x` lowering for non-literal unary minus.
//! Keeping two copies in sync was a maintenance hazard — this module
//! holds the single canonical body.
//!
//! Callers translate their own token type into [`ArithmeticToken`] via the
//! [`ArithmeticSource`] trait. The two callers differ in *what comes
//! around* arithmetic (a boolean parser also lexes `and`/`or`/`==`/...
//! in the same byte stream); they share *only* the arithmetic grammar
//! itself, which is what lives here.
//!
//! Errors are typed as [`ArithmeticError`] and re-mapped to each caller's
//! error enum at the call site.
//!
//! [`crate::expression`]: crate::expression
//! [`crate::boolean_expression`]: crate::boolean_expression

use cuentitos_common::{BinaryOperator, Expression, Value, VariableId};

/// The arithmetic sublanguage's token alphabet — payload-free so the
/// parser can pattern-match on it without copying identifier text.
/// Identifier and literal payloads are pulled out separately through
/// [`ArithmeticSource::take_ident`] and [`ArithmeticSource::take_int`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticTokenKind {
    Int,
    Bool,
    Ident,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

/// A payload-carrying arithmetic token. Provided for callers that
/// pre-tokenize into a `Vec` (see `SliceArithmeticSource`); the trait
/// itself works in terms of [`ArithmeticTokenKind`] plus
/// [`ArithmeticSource::take_int`]/[`ArithmeticSource::take_ident`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticToken {
    Int(u64),
    Bool(bool),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

impl ArithmeticToken {
    pub fn kind(&self) -> ArithmeticTokenKind {
        match self {
            ArithmeticToken::Int(_) => ArithmeticTokenKind::Int,
            ArithmeticToken::Bool(_) => ArithmeticTokenKind::Bool,
            ArithmeticToken::Ident(_) => ArithmeticTokenKind::Ident,
            ArithmeticToken::Plus => ArithmeticTokenKind::Plus,
            ArithmeticToken::Minus => ArithmeticTokenKind::Minus,
            ArithmeticToken::Star => ArithmeticTokenKind::Star,
            ArithmeticToken::Slash => ArithmeticTokenKind::Slash,
            ArithmeticToken::LParen => ArithmeticTokenKind::LParen,
            ArithmeticToken::RParen => ArithmeticTokenKind::RParen,
        }
    }
}

/// Errors produced by the shared arithmetic parser. Callers re-map these
/// into their own error enums.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithmeticError {
    /// The grammar didn't accept the input at the current position
    /// (e.g. a dangling operator, a stray comma, a primary that wasn't
    /// an integer / identifier / paren).
    Malformed,
    /// A literal exceeded the `i64` range. Carries the offending literal
    /// text including any leading sign.
    LiteralOverflow { literal: String },
    /// An identifier in primary position didn't resolve.
    UndefinedVariable { name: String },
    /// A `(` in primary position was never closed.
    UnbalancedParentheses,
    /// The source's recursion counter exceeded its cap. Raised from the
    /// shared body's two stack-growing descent points (unary `-`/`+` and
    /// the `(` branch of `primary`) so adversarial input — a long
    /// `----…x` chain or `(((…x))…)` nesting — can't drive the parser
    /// stack to overflow. Callers map this onto their own
    /// depth-exceeded variant.
    ExpressionTooDeep,
}

/// Bridges the caller's token stream and identifier scope to the
/// arithmetic parser. Implementations decide which of the caller's tokens
/// belong to the arithmetic sublanguage; anything else is reported as
/// `None` from [`ArithmeticSource::peek_kind`] so the parser stops
/// cleanly and the caller resumes from there.
///
/// The trait separates *discrimination* (cheap, `Copy`) from *payload
/// extraction* (allocating for `Ident`), so a recursive-descent parser
/// can `peek_kind` repeatedly at decision points without cloning the
/// identifier text every time.
pub trait ArithmeticSource {
    /// Look at the current arithmetic token's kind without consuming it.
    /// Returns `None` for end-of-stream **or** for any token that isn't
    /// part of the arithmetic sublanguage — the parser treats both
    /// identically (clean stop).
    fn peek_kind(&self) -> Option<ArithmeticTokenKind>;
    /// Consume the current token. Caller must have already observed it
    /// via [`peek_kind`](Self::peek_kind).
    fn advance(&mut self);
    /// Consume the current `Int` token and return its magnitude.
    /// Returns `None` if the cursor is not pointing at an `Int` (i.e.
    /// the caller forgot to guard with [`peek_kind`](Self::peek_kind));
    /// callers in this module `.expect()` on that since the surrounding
    /// `match` already verified the kind.
    fn take_int(&mut self) -> Option<u64>;
    /// Consume the current `Bool` token and return its value.
    /// Returns `None` if the cursor is not pointing at a `Bool` (i.e.
    /// the caller forgot to guard with [`peek_kind`](Self::peek_kind));
    /// callers in this module `.expect()` on that since the surrounding
    /// `match` already verified the kind.
    fn take_bool(&mut self) -> Option<bool>;
    /// Consume the current `Ident` token and return its text.
    /// Returns `None` if the cursor is not pointing at an `Ident` (i.e.
    /// the caller forgot to guard with [`peek_kind`](Self::peek_kind));
    /// callers in this module `.expect()` on that since the surrounding
    /// `match` already verified the kind.
    fn take_ident(&mut self) -> Option<String>;
    /// Resolve an identifier to a declared variable id.
    fn resolve(&self, name: &str) -> Option<VariableId>;
    /// Bump the source's recursion counter before a stack-growing
    /// descent (non-literal unary `-`/`+`, or the `(` branch of
    /// `primary`). Returns [`ArithmeticError::ExpressionTooDeep`] once
    /// the source's cap is exceeded. Pairs with
    /// [`leave_recursion`](Self::leave_recursion) on the success path;
    /// the error path leaks the bump because the surrounding parse is
    /// one-shot and the counter is dropped with the source.
    fn enter_recursion(&mut self) -> Result<(), ArithmeticError>;
    /// Decrement the recursion counter after a successful descent.
    fn leave_recursion(&mut self);
}

/// Parse an arithmetic expression from `stream`, leaving the cursor
/// positioned at the first token the arithmetic grammar didn't accept.
pub fn parse_arithmetic_expression<S: ArithmeticSource>(
    stream: &mut S,
) -> Result<Expression, ArithmeticError> {
    parse_additive(stream)
}

fn parse_additive<S: ArithmeticSource>(stream: &mut S) -> Result<Expression, ArithmeticError> {
    let mut left = parse_multiplicative(stream)?;
    loop {
        let operator = match stream.peek_kind() {
            Some(ArithmeticTokenKind::Plus) => BinaryOperator::Add,
            Some(ArithmeticTokenKind::Minus) => BinaryOperator::Subtract,
            _ => break,
        };
        stream.advance();
        let right = parse_multiplicative(stream)?;
        left = Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_multiplicative<S: ArithmeticSource>(
    stream: &mut S,
) -> Result<Expression, ArithmeticError> {
    let mut left = parse_unary(stream)?;
    loop {
        let operator = match stream.peek_kind() {
            Some(ArithmeticTokenKind::Star) => BinaryOperator::Multiply,
            Some(ArithmeticTokenKind::Slash) => BinaryOperator::Divide,
            _ => break,
        };
        stream.advance();
        let right = parse_unary(stream)?;
        left = Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_unary<S: ArithmeticSource>(stream: &mut S) -> Result<Expression, ArithmeticError> {
    match stream.peek_kind() {
        Some(ArithmeticTokenKind::Minus) => {
            stream.advance();
            // Fold `-` directly into a following literal so that
            // `i64::MIN` (whose magnitude doesn't fit in `i64`) is
            // representable.
            if let Some(ArithmeticTokenKind::Int) = stream.peek_kind() {
                let n = stream.take_int().expect("peek_kind guarded this");
                return negate_u64_literal(n).map(|v| Expression::Literal(Value::Integer(v)));
            }
            // Non-literal unary minus is the stack-growing case: the
            // following `parse_unary` call adds a frame for every `-`
            // in the chain. Bound it through the source so an adversarial
            // `----…x` can't drive the stack to overflow.
            stream.enter_recursion()?;
            let inner = parse_unary(stream)?;
            stream.leave_recursion();
            // Non-literal unary minus lowers to `0 - inner` so the
            // runtime's overflow-checked subtraction catches the
            // `-i64::MIN` edge case.
            Ok(Expression::Binary {
                operator: BinaryOperator::Subtract,
                left: Box::new(Expression::Literal(Value::Integer(0))),
                right: Box::new(inner),
            })
        }
        Some(ArithmeticTokenKind::Plus) => {
            stream.advance();
            // `+` unary is identity (no AST node), but the recursive
            // `parse_unary` call still grows the stack on `+++…`. Bound
            // it through the same counter as `-`.
            stream.enter_recursion()?;
            let inner = parse_unary(stream)?;
            stream.leave_recursion();
            Ok(inner)
        }
        _ => parse_primary(stream),
    }
}

fn parse_primary<S: ArithmeticSource>(stream: &mut S) -> Result<Expression, ArithmeticError> {
    match stream.peek_kind() {
        Some(ArithmeticTokenKind::Int) => {
            let n = stream.take_int().expect("peek_kind guarded this");
            if n > i64::MAX as u64 {
                return Err(ArithmeticError::LiteralOverflow {
                    literal: n.to_string(),
                });
            }
            Ok(Expression::Literal(Value::Integer(n as i64)))
        }
        Some(ArithmeticTokenKind::Bool) => {
            let value = stream.take_bool().expect("peek_kind guarded this");
            Ok(Expression::Literal(Value::Boolean(value)))
        }
        Some(ArithmeticTokenKind::Ident) => {
            let name = stream.take_ident().expect("peek_kind guarded this");
            match stream.resolve(&name) {
                Some(id) => Ok(Expression::Variable(id)),
                None => Err(ArithmeticError::UndefinedVariable { name }),
            }
        }
        Some(ArithmeticTokenKind::LParen) => {
            stream.advance();
            // Nested parens are the other stack-growing path: `(((((…)))))`
            // recurses through `parse_additive` once per `(`. Bound it
            // through the same counter as unary minus so `((((…x))))`
            // can't drive the stack to overflow.
            stream.enter_recursion()?;
            let inner = parse_additive(stream)?;
            stream.leave_recursion();
            match stream.peek_kind() {
                Some(ArithmeticTokenKind::RParen) => {
                    stream.advance();
                    Ok(inner)
                }
                _ => Err(ArithmeticError::UnbalancedParentheses),
            }
        }
        _ => Err(ArithmeticError::Malformed),
    }
}

/// Compute `-(n as i64)` without intermediate overflow. The `i64::MIN`
/// case is the only one whose absolute value doesn't fit in `i64`, so
/// it gets a direct return; anything larger overflows and the error
/// carries the original signed text.
fn negate_u64_literal(n: u64) -> Result<i64, ArithmeticError> {
    const ABS_MIN: u64 = (i64::MAX as u64) + 1;
    if n > ABS_MIN {
        return Err(ArithmeticError::LiteralOverflow {
            literal: format!("-{n}"),
        });
    }
    if n == ABS_MIN {
        return Ok(i64::MIN);
    }
    Ok(-(n as i64))
}
