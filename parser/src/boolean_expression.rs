//! Recursive-descent parser for `req` boolean expressions.
//!
//! Grammar (loosest precedence first):
//! ```text
//! or_expr    := and_expr (`or` and_expr)*
//! and_expr   := not_expr (`and` not_expr)*
//! not_expr   := `not` not_expr | primary
//! primary    := `(` or_expr `)` | comparison
//! comparison := arith_expr compare_op arith_expr
//! ```
//!
//! Arithmetic sub-expressions (the operands of comparisons) reuse the
//! standard `+ - * /` parser with their own parens. The boolean primary
//! disambiguates `(...)` between "parenthesized boolean group" and
//! "parenthesized arithmetic LHS of a comparison" by looking ahead through
//! the matched parenthesis to see whether a comparison operator follows.
//!
//! Errors are typed (see [`BooleanParseError`]) and carry enough context
//! for the caller to format the exact compatibility-test wording, e.g.
//! `Missing right operand for 'and' in 'req': 'x > 0 and'.`

use cuentitos_common::{
    BinaryOperator, BooleanExpression, ComparisonOperator, Expression, RequirementStatement, Value,
    VariableId,
};

/// Resolves identifiers (variable names) to declared [`VariableId`]s.
pub trait VariableResolver {
    fn resolve(&self, name: &str) -> Option<VariableId>;
}

impl<F: Fn(&str) -> Option<VariableId>> VariableResolver for F {
    fn resolve(&self, name: &str) -> Option<VariableId> {
        self(name)
    }
}

/// Parse `input` as a `req` boolean condition.
///
/// `input` is the trimmed text after the leading `req` keyword.
/// `resolver` maps identifiers in the condition to declared
/// [`VariableId`]s; unresolved names surface as
/// [`BooleanParseError::UndefinedVariable`].
pub fn parse_boolean_expression(
    input: &str,
    resolver: &dyn VariableResolver,
) -> Result<BooleanExpression, BooleanParseError> {
    let tokens = match tokenize(input) {
        Ok(tokens) => tokens,
        Err(TokenizeError::UnknownSymbol(symbol)) => {
            return Err(BooleanParseError::UnknownOperator { symbol });
        }
        Err(TokenizeError::LiteralOverflow(literal)) => {
            return Err(BooleanParseError::LiteralOverflow { literal });
        }
    };

    if tokens.is_empty() {
        return Err(BooleanParseError::Malformed);
    }

    let mut parser = BooleanParser {
        tokens: &tokens,
        position: 0,
        resolver,
    };

    let expression = parser.parse_or()?;
    if parser.position != parser.tokens.len() {
        // Unexpected trailing input. If a stray `)` is left over the
        // tokens still parsed individually but the boolean grammar
        // rejected them — surface as unbalanced parens.
        if matches!(parser.tokens.get(parser.position), Some(Token::RParen)) {
            return Err(BooleanParseError::UnbalancedParentheses);
        }
        // Leftover identifier after a complete expression usually means
        // the user wrote uppercase `AND`/`OR`/`NOT` expecting the logical
        // operator. Surface as `UndefinedVariable` so the message points
        // at the misspelled name.
        if let Some(Token::Ident(name)) = parser.tokens.get(parser.position) {
            if parser.resolver.resolve(name).is_none() {
                return Err(BooleanParseError::UndefinedVariable { name: name.clone() });
            }
        }
        return Err(BooleanParseError::Malformed);
    }
    Ok(expression)
}

/// Errors produced while parsing a `req` boolean expression. The caller
/// wraps these into [`crate::ParseError`] variants with file/line context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BooleanParseError {
    /// A bare integer expression was found as an operand (left or right)
    /// of `and`/`or`. The wrapping operator is known to the caller (e.g.
    /// `parse_and`). Carries the operator name.
    BareIntegerOperandOfLogical { operator: LogicalKeyword },
    /// A bare integer expression was the operand of `not`.
    BareIntegerOperandOfNot,
    /// A bare integer expression appeared at the top of a `req` condition
    /// with no surrounding logical operator. This is the legacy
    /// "Malformed expression in 'req'" case.
    BareIntegerAtTop,
    /// `and` or `or` appeared without a left operand.
    MissingLeftOperand { operator: LogicalKeyword },
    /// `and` or `or` appeared without a right operand.
    MissingRightOperand { operator: LogicalKeyword },
    /// `not` appeared without an operand.
    MissingNotOperand,
    /// Parentheses don't balance.
    UnbalancedParentheses,
    /// The RHS or LHS expression referenced an undeclared variable.
    UndefinedVariable { name: String },
    /// A symbol token was not a recognized operator. Carries the offending
    /// symbol so the caller can format `Unknown comparison operator: '~'`.
    UnknownOperator { symbol: String },
    /// Generic structural failure that doesn't fit a more specific case
    /// (e.g. missing RHS in a comparison, dangling arithmetic operator,
    /// stray identifier in a position requiring an operator).
    Malformed,
    /// A literal exceeded the i64 range at parse time. Carries the
    /// offending literal text (e.g. `99999999999999999999`) so the caller
    /// can surface it in the error message.
    LiteralOverflow { literal: String },
}

/// A logical-operator keyword name. Stored as an enum (rather than a raw
/// string) so the formatter has a single source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalKeyword {
    And,
    Or,
}

impl LogicalKeyword {
    pub fn as_str(self) -> &'static str {
        match self {
            LogicalKeyword::And => "and",
            LogicalKeyword::Or => "or",
        }
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Int(u64),
    Ident(String),
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    LParen,
    RParen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenizeError {
    UnknownSymbol(String),
    LiteralOverflow(String),
}

fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if (c as char).is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            b'+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            b'-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            b'*' => {
                tokens.push(Token::Star);
                i += 1;
            }
            b'/' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            b'(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            b')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token::GreaterOrEqual);
                    i += 2;
                } else {
                    tokens.push(Token::Greater);
                    i += 1;
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token::LessOrEqual);
                    i += 2;
                } else {
                    tokens.push(Token::Less);
                    i += 1;
                }
            }
            b'=' => {
                tokens.push(Token::Equal);
                i += 1;
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    tokens.push(Token::NotEqual);
                    i += 2;
                } else {
                    return Err(TokenizeError::UnknownSymbol("!".to_string()));
                }
            }
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let literal =
                    std::str::from_utf8(&bytes[start..i]).expect("ascii digits are valid utf8");
                // u64::from_str only fails here for magnitudes greater than
                // u64::MAX — every literal in range already parses. Surface
                // that as LiteralOverflow so the user sees the same message
                // as i64-range overflows caught later in the parser.
                let parsed: u64 = literal
                    .parse()
                    .map_err(|_| TokenizeError::LiteralOverflow(literal.to_string()))?;
                tokens.push(Token::Int(parsed));
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let name =
                    std::str::from_utf8(&bytes[start..i]).expect("identifier chars are ascii");
                let token = match name {
                    "and" => Token::LogicalAnd,
                    "or" => Token::LogicalOr,
                    "not" => Token::LogicalNot,
                    _ => Token::Ident(name.to_string()),
                };
                tokens.push(token);
            }
            _ => {
                let start = i;
                while i < bytes.len() {
                    let c = bytes[i];
                    if (c as char).is_whitespace()
                        || c.is_ascii_alphanumeric()
                        || c == b'_'
                        || c == b'('
                        || c == b')'
                    {
                        break;
                    }
                    i += 1;
                }
                let symbol = std::str::from_utf8(&bytes[start..i])
                    .unwrap_or("?")
                    .to_string();
                return Err(TokenizeError::UnknownSymbol(symbol));
            }
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Recursive-descent parser
// ---------------------------------------------------------------------------

struct BooleanParser<'a> {
    tokens: &'a [Token],
    position: usize,
    resolver: &'a dyn VariableResolver,
}

impl<'a> BooleanParser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn parse_or(&mut self) -> Result<BooleanExpression, BooleanParseError> {
        // Detect missing left operand for `or` — first token is `or` itself.
        if matches!(self.peek(), Some(Token::LogicalOr)) {
            return Err(BooleanParseError::MissingLeftOperand {
                operator: LogicalKeyword::Or,
            });
        }
        let mut left = self.parse_and(LogicalContext::OuterOrTop)?;
        while matches!(self.peek(), Some(Token::LogicalOr)) {
            self.position += 1;
            if self.peek().is_none() || matches!(self.peek(), Some(Token::RParen)) {
                return Err(BooleanParseError::MissingRightOperand {
                    operator: LogicalKeyword::Or,
                });
            }
            let right = self.parse_and(LogicalContext::RightOfOr)?;
            left = BooleanExpression::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(
        &mut self,
        context: LogicalContext,
    ) -> Result<BooleanExpression, BooleanParseError> {
        if matches!(self.peek(), Some(Token::LogicalAnd)) {
            return Err(BooleanParseError::MissingLeftOperand {
                operator: LogicalKeyword::And,
            });
        }
        let mut left = self.parse_not(context)?;
        while matches!(self.peek(), Some(Token::LogicalAnd)) {
            self.position += 1;
            if self.peek().is_none() || matches!(self.peek(), Some(Token::RParen)) {
                return Err(BooleanParseError::MissingRightOperand {
                    operator: LogicalKeyword::And,
                });
            }
            let right = self.parse_not(LogicalContext::RightOfAnd)?;
            left = BooleanExpression::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(
        &mut self,
        context: LogicalContext,
    ) -> Result<BooleanExpression, BooleanParseError> {
        if matches!(self.peek(), Some(Token::LogicalNot)) {
            self.position += 1;
            // Missing operand entirely.
            if self.peek().is_none() || matches!(self.peek(), Some(Token::RParen)) {
                return Err(BooleanParseError::MissingNotOperand);
            }
            let inner = self.parse_not(LogicalContext::OperandOfNot)?;
            return Ok(BooleanExpression::Not(Box::new(inner)));
        }
        self.parse_primary(context)
    }

    fn parse_primary(
        &mut self,
        context: LogicalContext,
    ) -> Result<BooleanExpression, BooleanParseError> {
        if matches!(self.peek(), Some(Token::LParen)) {
            // Disambiguate: is this a parenthesized boolean group, or the
            // parenthesized LHS of a comparison? The lookahead surfaces
            // unbalanced-paren errors immediately at the open `(`, instead
            // of routing into one branch and discovering the imbalance
            // partway through parsing.
            if self.parens_open_arithmetic_lhs()? {
                return self.parse_comparison(context);
            }
            self.position += 1;
            let inner = self.parse_or()?;
            match self.peek() {
                Some(Token::RParen) => {
                    self.position += 1;
                    Ok(inner)
                }
                _ => Err(BooleanParseError::UnbalancedParentheses),
            }
        } else {
            self.parse_comparison(context)
        }
    }

    /// Look ahead from a `(` at `self.position`: scan to its matching `)`,
    /// then check whether the next non-whitespace token is a comparison
    /// operator. If so, the `(` is part of an arithmetic LHS like
    /// `(a + b) > 0`. Otherwise it's a boolean group.
    ///
    /// Returns `Err(UnbalancedParentheses)` if the open `(` is never
    /// closed — surfacing the error at the open paren rather than
    /// routing into the boolean-group branch and discovering it later.
    fn parens_open_arithmetic_lhs(&self) -> Result<bool, BooleanParseError> {
        debug_assert!(matches!(
            self.tokens.get(self.position),
            Some(Token::LParen)
        ));
        let mut depth = 0_usize;
        let mut i = self.position;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        // Matching paren found. Look at next token.
                        return Ok(matches!(
                            self.tokens.get(i + 1),
                            Some(
                                Token::Equal
                                    | Token::NotEqual
                                    | Token::Less
                                    | Token::LessOrEqual
                                    | Token::Greater
                                    | Token::GreaterOrEqual
                            )
                        ));
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Err(BooleanParseError::UnbalancedParentheses)
    }

    fn parse_comparison(
        &mut self,
        context: LogicalContext,
    ) -> Result<BooleanExpression, BooleanParseError> {
        let left = self.parse_arith()?;
        let comparison_operator = match self.peek() {
            Some(Token::Equal) => Some(ComparisonOperator::Equal),
            Some(Token::NotEqual) => Some(ComparisonOperator::NotEqual),
            Some(Token::Less) => Some(ComparisonOperator::Less),
            Some(Token::LessOrEqual) => Some(ComparisonOperator::LessOrEqual),
            Some(Token::Greater) => Some(ComparisonOperator::Greater),
            Some(Token::GreaterOrEqual) => Some(ComparisonOperator::GreaterOrEqual),
            _ => None,
        };
        if let Some(operator) = comparison_operator {
            self.position += 1;
            let right = self.parse_arith()?;
            return Ok(BooleanExpression::Comparison(RequirementStatement::new(
                left, operator, right,
            )));
        }
        // No comparison operator — we got an arithmetic expression where
        // a boolean was expected. Surface a context-aware error.
        match self.peek() {
            Some(Token::LogicalAnd) => Err(BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::And,
            }),
            Some(Token::LogicalOr) => Err(BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::Or,
            }),
            Some(Token::RParen) | None => match context {
                LogicalContext::OperandOfNot => Err(BooleanParseError::BareIntegerOperandOfNot),
                LogicalContext::RightOfAnd => Err(BooleanParseError::BareIntegerOperandOfLogical {
                    operator: LogicalKeyword::And,
                }),
                LogicalContext::RightOfOr => Err(BooleanParseError::BareIntegerOperandOfLogical {
                    operator: LogicalKeyword::Or,
                }),
                LogicalContext::OuterOrTop => Err(BooleanParseError::BareIntegerAtTop),
            },
            _ => Err(BooleanParseError::Malformed),
        }
    }

    // ----- Arithmetic sub-language -----
    //
    // Mirrors the recursive descent in `parser/src/expression.rs`:
    // precedence `+ -` < `* /` < unary `-` < primary, with the same
    // `i64::MIN`-aware literal negation. **DO NOT change operator
    // precedence, integer-literal handling, or unary-minus rules here
    // without mirroring the change in `parser/src/expression.rs`.**
    // Tracked as a follow-up: extract a single arithmetic helper that
    // both call into.

    fn parse_arith(&mut self) -> Result<Expression, BooleanParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expression, BooleanParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let operator = match self.peek() {
                Some(Token::Plus) => BinaryOperator::Add,
                Some(Token::Minus) => BinaryOperator::Subtract,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, BooleanParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let operator = match self.peek() {
                Some(Token::Star) => BinaryOperator::Multiply,
                Some(Token::Slash) => BinaryOperator::Divide,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_unary()?;
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, BooleanParseError> {
        match self.peek() {
            Some(Token::Minus) => {
                self.position += 1;
                if let Some(Token::Int(n)) = self.peek() {
                    let n = *n;
                    self.position += 1;
                    return negate_u64_literal(n).map(|v| Expression::Literal(Value::Integer(v)));
                }
                let inner = self.parse_unary()?;
                Ok(Expression::Binary {
                    operator: BinaryOperator::Subtract,
                    left: Box::new(Expression::Literal(Value::Integer(0))),
                    right: Box::new(inner),
                })
            }
            Some(Token::Plus) => {
                self.position += 1;
                self.parse_unary()
            }
            _ => self.parse_arith_primary(),
        }
    }

    fn parse_arith_primary(&mut self) -> Result<Expression, BooleanParseError> {
        match self.peek() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.position += 1;
                if n > i64::MAX as u64 {
                    return Err(BooleanParseError::LiteralOverflow {
                        literal: n.to_string(),
                    });
                }
                Ok(Expression::Literal(Value::Integer(n as i64)))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.position += 1;
                match self.resolver.resolve(&name) {
                    Some(id) => Ok(Expression::Variable(id)),
                    None => Err(BooleanParseError::UndefinedVariable { name }),
                }
            }
            Some(Token::LParen) => {
                self.position += 1;
                let inner = self.parse_additive()?;
                match self.peek() {
                    Some(Token::RParen) => {
                        self.position += 1;
                        Ok(inner)
                    }
                    _ => Err(BooleanParseError::UnbalancedParentheses),
                }
            }
            _ => Err(BooleanParseError::Malformed),
        }
    }
}

/// Tracks where the current parse was invoked from so that the error
/// surfaced when a bare arithmetic expression appears in a comparison
/// position can name the surrounding logical operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogicalContext {
    OuterOrTop,
    RightOfAnd,
    RightOfOr,
    OperandOfNot,
}

fn negate_u64_literal(n: u64) -> Result<i64, BooleanParseError> {
    const ABS_MIN: u64 = (i64::MAX as u64) + 1;
    if n > ABS_MIN {
        return Err(BooleanParseError::LiteralOverflow {
            literal: format!("-{n}"),
        });
    }
    if n == ABS_MIN {
        return Ok(i64::MIN);
    }
    Ok(-(n as i64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_resolver(pairs: &[(&str, VariableId)]) -> impl VariableResolver {
        let map: HashMap<String, VariableId> =
            pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        move |name: &str| map.get(name).copied()
    }

    fn parse(
        input: &str,
        vars: &[(&str, VariableId)],
    ) -> Result<BooleanExpression, BooleanParseError> {
        let resolver = make_resolver(vars);
        parse_boolean_expression(input, &resolver)
    }

    #[test]
    fn parses_single_comparison() {
        let result = parse("x > 0", &[("x", 0)]).unwrap();
        match result {
            BooleanExpression::Comparison(stmt) => {
                assert_eq!(stmt.operator, ComparisonOperator::Greater);
            }
            _ => panic!("expected Comparison"),
        }
    }

    #[test]
    fn parses_and() {
        let result = parse("x > 0 and y > 0", &[("x", 0), ("y", 1)]).unwrap();
        assert!(matches!(result, BooleanExpression::And(_, _)));
    }

    #[test]
    fn parses_or() {
        let result = parse("x > 0 or y > 0", &[("x", 0), ("y", 1)]).unwrap();
        assert!(matches!(result, BooleanExpression::Or(_, _)));
    }

    #[test]
    fn parses_not() {
        let result = parse("not x > 0", &[("x", 0)]).unwrap();
        assert!(matches!(result, BooleanExpression::Not(_)));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        // a or b and c should parse as a or (b and c)
        let result = parse("a > 0 or b > 0 and c > 0", &[("a", 0), ("b", 1), ("c", 2)]).unwrap();
        match result {
            BooleanExpression::Or(_, right) => {
                assert!(matches!(*right, BooleanExpression::And(_, _)));
            }
            _ => panic!("expected Or at top, got {:?}", result),
        }
    }

    #[test]
    fn parens_override_precedence() {
        // (a or b) and c
        let result = parse(
            "(a > 0 or b > 0) and c > 0",
            &[("a", 0), ("b", 1), ("c", 2)],
        )
        .unwrap();
        match result {
            BooleanExpression::And(left, _) => {
                assert!(matches!(*left, BooleanExpression::Or(_, _)));
            }
            _ => panic!("expected And at top"),
        }
    }

    #[test]
    fn not_binds_tighter_than_and() {
        // not a > 0 and b > 0 → (not (a > 0)) and (b > 0)
        let result = parse("not a > 0 and b > 0", &[("a", 0), ("b", 1)]).unwrap();
        match result {
            BooleanExpression::And(left, _) => {
                assert!(matches!(*left, BooleanExpression::Not(_)));
            }
            _ => panic!("expected And at top"),
        }
    }

    #[test]
    fn arithmetic_lhs_with_parens() {
        // (a + b) > 0 — paren is part of arith LHS
        let result = parse("(a + b) > 0", &[("a", 0), ("b", 1)]).unwrap();
        match result {
            BooleanExpression::Comparison(stmt) => {
                assert_eq!(stmt.operator, ComparisonOperator::Greater);
                assert!(matches!(stmt.left, Expression::Binary { .. }));
            }
            _ => panic!("expected Comparison"),
        }
    }

    #[test]
    fn rejects_bare_integer_left_of_and() {
        let err = parse("health and shield > 0", &[("health", 0), ("shield", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_left_of_or() {
        let err = parse("health or shield > 0", &[("health", 0), ("shield", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::Or,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_right_of_and() {
        let err = parse("x > 0 and y", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_right_of_or() {
        let err = parse("x > 0 or y", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::BareIntegerOperandOfLogical {
                operator: LogicalKeyword::Or,
            }
        );
    }

    #[test]
    fn rejects_bare_integer_operand_of_not() {
        let err = parse("not health", &[("health", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::BareIntegerOperandOfNot);
    }

    #[test]
    fn rejects_missing_left_operand_and() {
        let err = parse("and x > 0", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::MissingLeftOperand {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_missing_left_operand_or() {
        let err = parse("or x > 0", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::MissingLeftOperand {
                operator: LogicalKeyword::Or,
            }
        );
    }

    #[test]
    fn rejects_missing_right_operand_and() {
        let err = parse("x > 0 and", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::MissingRightOperand {
                operator: LogicalKeyword::And,
            }
        );
    }

    #[test]
    fn rejects_missing_right_operand_or() {
        let err = parse("x > 0 or", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::MissingRightOperand {
                operator: LogicalKeyword::Or,
            }
        );
    }

    #[test]
    fn rejects_missing_not_operand() {
        let err = parse("not", &[]).unwrap_err();
        assert_eq!(err, BooleanParseError::MissingNotOperand);
    }

    #[test]
    fn rejects_unbalanced_close_paren() {
        let err = parse("x > 0 and x < 10)", &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::UnbalancedParentheses);
    }

    #[test]
    fn rejects_unbalanced_open_paren() {
        let err = parse("(x > 0 and x < 10", &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::UnbalancedParentheses);
    }

    #[test]
    fn uppercase_and_is_identifier() {
        let err = parse("x > 0 AND y > 0", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::UndefinedVariable {
                name: "AND".to_string()
            }
        );
    }

    #[test]
    fn uppercase_or_is_identifier() {
        let err = parse("x > 0 OR y > 0", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::UndefinedVariable {
                name: "OR".to_string()
            }
        );
    }

    #[test]
    fn uppercase_not_is_identifier() {
        let err = parse("x > 0 and NOT y > 0", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::UndefinedVariable {
                name: "NOT".to_string()
            }
        );
    }

    #[test]
    fn unknown_operator_tilde() {
        let err = parse("x ~ 5", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::UnknownOperator {
                symbol: "~".to_string()
            }
        );
    }

    #[test]
    fn malformed_dangling_arithmetic_operator() {
        let err = parse("x > 5 +", &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::Malformed);
    }

    #[test]
    fn literal_overflow_carries_the_offending_literal() {
        let err = parse("x > 99999999999999999999", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::LiteralOverflow {
                literal: "99999999999999999999".to_string()
            }
        );
    }

    #[test]
    fn negative_literal_overflow_carries_the_offending_literal() {
        // `-9223372036854775809` = -(i64::MAX + 2). The magnitude fits in
        // u64 so the tokenizer accepts it as Int, then negate_u64_literal
        // catches the overflow and stamps the sign onto the reported
        // literal.
        let err = parse("x > -9223372036854775809", &[("x", 0)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::LiteralOverflow {
                literal: "-9223372036854775809".to_string()
            }
        );
    }
}
