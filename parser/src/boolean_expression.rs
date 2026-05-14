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
//! Arithmetic operands of comparisons are parsed by the shared body in
//! [`crate::arithmetic`]; this module's [`ArithmeticSource`] impl on
//! [`BooleanParser`] projects boolean tokens into that body. The boolean
//! primary disambiguates `(...)` between "parenthesized boolean group"
//! and "parenthesized arithmetic LHS of a comparison" by looking ahead
//! through the matched parenthesis to see whether a comparison operator
//! follows.
//!
//! Errors are typed (see [`BooleanParseError`]) and carry enough context
//! for the caller to format the exact compatibility-test wording, e.g.
//! `Missing right operand for 'and' in 'req': 'x > 0 and'.`

use cuentitos_common::{
    BooleanExpression, ComparisonOperator, Expression, RequirementStatement, VariableId,
};

use crate::arithmetic::{
    parse_arithmetic_expression, ArithmeticError, ArithmeticSource, ArithmeticTokenKind,
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
            return Err(BooleanParseError::UnknownSymbol { symbol });
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
        depth: 0,
    };

    let expression = parser.parse_or()?;
    // Every enter_recursion on the success path is paired with a
    // leave_recursion. If we ever land here with non-zero depth, a new
    // descent point was added without a matching leave.
    debug_assert_eq!(parser.depth, 0, "depth bumps unbalanced on success");
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
    /// A symbol that the tokenizer didn't recognize as part of any
    /// `req`-grammar token. Carries the offending lexeme so the caller
    /// can format `Unknown operator '&' in 'req'`. Includes things that
    /// aren't comparison operators at all (e.g. `&`, `|`, `~`), so the
    /// message must not over-claim "comparison operator".
    UnknownSymbol { symbol: String },
    /// Generic structural failure that doesn't fit a more specific case
    /// (e.g. missing RHS in a comparison, dangling arithmetic operator,
    /// stray identifier in a position requiring an operator).
    Malformed,
    /// A literal exceeded the i64 range at parse time. Carries the
    /// offending literal text (e.g. `99999999999999999999`) so the caller
    /// can surface it in the error message.
    LiteralOverflow { literal: String },
    /// The boolean expression nests deeper than [`MAX_EXPRESSION_DEPTH`].
    /// Capping parse depth bounds validation and evaluation depth too —
    /// see [`MAX_EXPRESSION_DEPTH`] for the rationale.
    ExpressionTooDeep,
}

/// Maximum nesting depth accepted in a `req` boolean expression. Each
/// `not`, parenthesized group, or other descent below a logical level
/// contributes one. The cap reflects AST nesting depth, not parser
/// function-call frames: a single comparison costs 0, `not x > 0` costs
/// 1, `(x > 0)` costs 1, `not not x > 0` costs 2. Long chains of
/// `and`/`or` are parsed iteratively and don't contribute.
///
/// Set deliberately tight at 64 because real `req` conditions in a
/// narrative script max out around 4–5 levels; the cap exists to bound
/// adversarial input, and a tighter bound bounds CPU/memory consumed
/// before the parser bails out. If a legitimate use case ever needs
/// more, raise this rather than removing the cap entirely.
pub const MAX_EXPRESSION_DEPTH: usize = 64;

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
            '>' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::GreaterOrEqual);
                } else {
                    tokens.push(Token::Greater);
                }
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::LessOrEqual);
                } else {
                    tokens.push(Token::Less);
                }
            }
            '=' => {
                chars.next();
                tokens.push(Token::Equal);
            }
            '!' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::NotEqual);
                } else {
                    return Err(TokenizeError::UnknownSymbol("!".to_string()));
                }
            }
            c if c.is_ascii_digit() => {
                let mut literal = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        literal.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // u64::from_str only fails here for magnitudes greater than
                // u64::MAX — every literal in range already parses. Surface
                // that as LiteralOverflow so the user sees the same message
                // as i64-range overflows caught later in the parser.
                let parsed: u64 = literal
                    .parse()
                    .map_err(|_| TokenizeError::LiteralOverflow(literal.clone()))?;
                tokens.push(Token::Int(parsed));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match name.as_str() {
                    "and" => Token::LogicalAnd,
                    "or" => Token::LogicalOr,
                    "not" => Token::LogicalNot,
                    _ => Token::Ident(name),
                };
                tokens.push(token);
            }
            _ => {
                // Accumulate a contiguous run of symbol characters so the
                // error names the full offending lexeme (e.g. `~~`, not just
                // `~`). Walking by chars keeps multi-byte UTF-8 symbols
                // intact in the reported text.
                let mut symbol = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace()
                        || c.is_ascii_alphanumeric()
                        || c == '_'
                        || c == '('
                        || c == ')'
                    {
                        break;
                    }
                    symbol.push(c);
                    chars.next();
                }
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
    /// Current AST nesting depth. Bumped only when descending past a
    /// `not` or `(` into a recursive call; not bumped for plain comparison
    /// leaves or for parse-function entries that don't descend. On the
    /// error path the depth leaks — fine because the parse short-circuits
    /// and the source is one-shot. Bounded by [`MAX_EXPRESSION_DEPTH`].
    depth: usize,
}

impl<'a> BooleanParser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Enter a recursive frame. Returns `ExpressionTooDeep` once the
    /// running tree depth exceeds the cap, before any further allocation.
    fn enter_recursion(&mut self) -> Result<(), BooleanParseError> {
        self.depth += 1;
        if self.depth > MAX_EXPRESSION_DEPTH {
            return Err(BooleanParseError::ExpressionTooDeep);
        }
        Ok(())
    }

    fn leave_recursion(&mut self) {
        self.depth -= 1;
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
            // Bump only at the actual descent — one `not` wrap = one AST
            // level. A bare comparison (the else branch) doesn't bump.
            self.enter_recursion()?;
            let inner = self.parse_not(LogicalContext::OperandOfNot)?;
            self.leave_recursion();
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
            // Bump only when actually descending into a paren group.
            self.enter_recursion()?;
            let inner = self.parse_or()?;
            match self.peek() {
                Some(Token::RParen) => {
                    self.position += 1;
                    self.leave_recursion();
                    Ok(inner)
                }
                _ => Err(BooleanParseError::UnbalancedParentheses),
            }
        } else {
            self.parse_comparison(context)
        }
    }

    /// Look ahead from a `(` at `self.position`: scan to its matching
    /// `)`, then keep walking past any arithmetic continuation tokens
    /// (more arith operators, operands, and balanced nested paren
    /// groups). If the first non-arithmetic token at depth 0 is a
    /// comparison operator, the `(` opens an arithmetic LHS like
    /// `(a + b) > 0` or `(a + b) + 1 > 0`. Otherwise it's a boolean
    /// group.
    ///
    /// Returns `Err(UnbalancedParentheses)` if the open `(` is never
    /// closed — surfacing the error at the open paren rather than
    /// routing into the boolean-group branch and discovering it later.
    //
    // Each open paren triggers a forward scan to its matching `)` plus
    // a continuation walk; nesting compounds to O(d²) in the worst case
    // (`((((a > 0))))` does d full scans, each O(d)). Real `req`
    // conditions stay shallow so this is a non-issue today.
    //
    // TODO: precompute paren-match positions during tokenization
    // (`Vec<usize>` indexed by paren index) if `req` expressions ever
    // grow large enough that the per-`(` O(n) scan dominates.
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
                        // Matching paren of the leading `(` found. Skip
                        // past anything that could continue the same
                        // arithmetic expression and decide based on what
                        // comes after.
                        return Ok(self.is_arithmetic_continuation_then_comparison(i + 1));
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Err(BooleanParseError::UnbalancedParentheses)
    }

    /// Starting at `start`, walk forward through tokens that could
    /// continue an arithmetic expression (`+`/`-`/`*`/`/`, integer
    /// literals, identifiers, and balanced nested paren groups). Return
    /// `true` iff the first token outside that continuation, at top
    /// depth, is a comparison operator.
    ///
    /// Used by [`parens_open_arithmetic_lhs`] to disambiguate a leading
    /// `(` that opens an arithmetic LHS (`(a + b) + 1 > c`) from one
    /// that opens a boolean group (`(a > 0) and b > 0`).
    fn is_arithmetic_continuation_then_comparison(&self, start: usize) -> bool {
        let mut i = start;
        let mut depth = 0_usize;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Equal
                | Token::NotEqual
                | Token::Less
                | Token::LessOrEqual
                | Token::Greater
                | Token::GreaterOrEqual
                    if depth == 0 =>
                {
                    return true;
                }
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Int(_)
                | Token::Ident(_) => {}
                Token::LParen => depth += 1,
                Token::RParen => {
                    if depth == 0 {
                        // Stray close paren outside the LHS — not part
                        // of the same arithmetic expression. Hand back
                        // to the boolean grammar so the imbalance
                        // surfaces at the right spot.
                        return false;
                    }
                    depth -= 1;
                }
                // Logical keywords or comparison ops below a nested
                // paren group: not an arithmetic LHS.
                _ => return false,
            }
            i += 1;
        }
        false
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

    // The arithmetic sublanguage (operands of comparisons) is parsed by
    // the shared body in [`crate::arithmetic`]; this struct's
    // [`ArithmeticSource`] impl below projects the boolean tokens into
    // [`ArithmeticTokenKind`] and the call site maps [`ArithmeticError`]
    // back into [`BooleanParseError`].

    fn parse_arith(&mut self) -> Result<Expression, BooleanParseError> {
        parse_arithmetic_expression(self).map_err(map_arithmetic_error)
    }
}

impl<'a> ArithmeticSource for BooleanParser<'a> {
    fn peek_kind(&self) -> Option<ArithmeticTokenKind> {
        match self.tokens.get(self.position)? {
            Token::Int(_) => Some(ArithmeticTokenKind::Int),
            Token::Ident(_) => Some(ArithmeticTokenKind::Ident),
            Token::Plus => Some(ArithmeticTokenKind::Plus),
            Token::Minus => Some(ArithmeticTokenKind::Minus),
            Token::Star => Some(ArithmeticTokenKind::Star),
            Token::Slash => Some(ArithmeticTokenKind::Slash),
            Token::LParen => Some(ArithmeticTokenKind::LParen),
            Token::RParen => Some(ArithmeticTokenKind::RParen),
            // Logical and comparison tokens aren't part of the arithmetic
            // sublanguage — surface them as end-of-stream so the shared
            // parser stops cleanly and the boolean parser resumes from
            // here.
            _ => None,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn take_int(&mut self) -> Option<u64> {
        let Token::Int(n) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = *n;
        self.position += 1;
        Some(value)
    }

    fn take_ident(&mut self) -> Option<String> {
        let Token::Ident(name) = self.tokens.get(self.position)? else {
            return None;
        };
        let value = name.clone();
        self.position += 1;
        Some(value)
    }

    fn resolve(&self, name: &str) -> Option<VariableId> {
        self.resolver.resolve(name)
    }

    // Trait-side recursion bookkeeping. Reuses the boolean parser's
    // `depth` field so boolean nesting (`not`, `(...)` boolean groups)
    // and arithmetic nesting (`---x`, `((x))` arith LHS) share a single
    // [`MAX_EXPRESSION_DEPTH`] budget — adversarial input can't dodge
    // the cap by switching layers.
    //
    // The inherent `enter_recursion`/`leave_recursion` methods on
    // [`BooleanParser`] surface the boolean error type and stay
    // selected at boolean-layer call sites by Rust's preference for
    // inherent methods; this trait impl is reached only via generic
    // dispatch from the shared arithmetic body.
    fn enter_recursion(&mut self) -> Result<(), ArithmeticError> {
        self.depth += 1;
        if self.depth > MAX_EXPRESSION_DEPTH {
            return Err(ArithmeticError::ExpressionTooDeep);
        }
        Ok(())
    }

    fn leave_recursion(&mut self) {
        self.depth -= 1;
    }
}

fn map_arithmetic_error(error: ArithmeticError) -> BooleanParseError {
    match error {
        ArithmeticError::Malformed => BooleanParseError::Malformed,
        ArithmeticError::LiteralOverflow { literal } => {
            BooleanParseError::LiteralOverflow { literal }
        }
        ArithmeticError::UndefinedVariable { name } => {
            BooleanParseError::UndefinedVariable { name }
        }
        ArithmeticError::UnbalancedParentheses => BooleanParseError::UnbalancedParentheses,
        // Fold into the existing boolean depth-cap error variant — both
        // boolean and arithmetic nesting share one budget, so one
        // diagnostic suffices.
        ArithmeticError::ExpressionTooDeep => BooleanParseError::ExpressionTooDeep,
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
    fn arithmetic_lhs_with_parens_then_more_arithmetic() {
        // (a + b) + 1 > c — leading paren opens arith LHS, then `+ 1`
        // continues the same arithmetic expression before the comparison.
        let result = parse("(a + b) + 1 > c", &[("a", 0), ("b", 1), ("c", 2)]).unwrap();
        match result {
            BooleanExpression::Comparison(stmt) => {
                assert_eq!(stmt.operator, ComparisonOperator::Greater);
                assert!(matches!(stmt.left, Expression::Binary { .. }));
                assert_eq!(stmt.right, Expression::Variable(2));
            }
            _ => panic!("expected Comparison"),
        }
    }

    #[test]
    fn arithmetic_lhs_with_parens_then_multiplication() {
        // (a + b) * 2 > c — same shape, multiplicative continuation.
        let result = parse("(a + b) * 2 > c", &[("a", 0), ("b", 1), ("c", 2)]).unwrap();
        assert!(matches!(result, BooleanExpression::Comparison(_)));
    }

    #[test]
    fn arithmetic_lhs_with_single_var_in_parens_then_more() {
        // (a) + 1 > 0 — degenerate single-variable paren followed by arith.
        let result = parse("(a) + 1 > 0", &[("a", 0)]).unwrap();
        assert!(matches!(result, BooleanExpression::Comparison(_)));
    }

    #[test]
    fn arithmetic_lhs_with_nested_parens_then_more() {
        // ((a + b)) + 1 > 0 — nested parens, then continuation.
        let result = parse("((a + b)) + 1 > 0", &[("a", 0), ("b", 1)]).unwrap();
        assert!(matches!(result, BooleanExpression::Comparison(_)));
    }

    #[test]
    fn boolean_group_followed_by_logical_still_parses_as_group() {
        // (a > 0) and b > 0 — leading paren opens a boolean group, not
        // an arith LHS. The continuation scanner sees `and` after the
        // matching `)` and falls back to the boolean grammar.
        let result = parse("(a > 0) and b > 0", &[("a", 0), ("b", 1)]).unwrap();
        assert!(matches!(result, BooleanExpression::And(_, _)));
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
            BooleanParseError::UnknownSymbol {
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
    fn rejects_expression_nested_beyond_cap() {
        // A chain of `not` deeper than MAX_EXPRESSION_DEPTH must be
        // rejected at parse time, before we recurse far enough to risk
        // stack overflow. The condition `x > 0` is well-formed by
        // itself; only the wrapping depth makes the input bad.
        let mut input = String::new();
        for _ in 0..=MAX_EXPRESSION_DEPTH {
            input.push_str("not ");
        }
        input.push_str("x > 0");
        let err = parse(&input, &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::ExpressionTooDeep);
    }

    #[test]
    fn accepts_expression_at_modest_depth() {
        // Sanity check: a depth well under the cap still parses.
        let mut input = String::new();
        for _ in 0..16 {
            input.push_str("not ");
        }
        input.push_str("x > 0");
        let result = parse(&input, &[("x", 0)]).unwrap();
        // Outer node is `Not` — the chain reduces by one each level.
        assert!(matches!(result, BooleanExpression::Not(_)));
    }

    #[test]
    fn rejects_deep_arithmetic_unary_minus() {
        // 200 leading `-`s on the RHS recurse through the shared
        // arithmetic `parse_unary` body. Before the arith side joined
        // the cap, this used to grow the parser stack linearly with the
        // input length. Now the same `MAX_EXPRESSION_DEPTH` budget
        // catches it before recursion gets dangerous.
        let mut input = String::from("x > ");
        for _ in 0..200 {
            input.push('-');
        }
        input.push('1');
        let err = parse(&input, &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::ExpressionTooDeep);
    }

    #[test]
    fn rejects_deep_arithmetic_paren_nesting() {
        // 200 nested `(`s in the RHS recurse through the shared
        // arithmetic `parse_primary` LParen branch. Same class of
        // stack-growth bug as the unary-minus chain; same cap catches
        // it. Closing parens are emitted symmetrically so any error
        // we observe is depth-related, not malformed-input-related.
        let mut input = String::from("x > ");
        for _ in 0..200 {
            input.push('(');
        }
        input.push('1');
        for _ in 0..200 {
            input.push(')');
        }
        let err = parse(&input, &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::ExpressionTooDeep);
    }

    #[test]
    fn boolean_and_arithmetic_depth_share_a_budget() {
        // Combining `not` wraps with an arith paren-nested RHS must
        // still cap at the same `MAX_EXPRESSION_DEPTH` — boolean and
        // arithmetic nesting feed the same counter, so an adversarial
        // input can't switch layers to dodge the limit.
        let half_minus_one = MAX_EXPRESSION_DEPTH / 2;
        let mut input = String::new();
        for _ in 0..half_minus_one {
            input.push_str("not ");
        }
        input.push_str("x > ");
        for _ in 0..=half_minus_one {
            input.push('(');
        }
        input.push('1');
        for _ in 0..=half_minus_one {
            input.push(')');
        }
        let err = parse(&input, &[("x", 0)]).unwrap_err();
        assert_eq!(err, BooleanParseError::ExpressionTooDeep);
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

    #[test]
    fn unknown_operator_preserves_non_ascii_symbol() {
        // U+2227 LOGICAL AND. The tokenizer walks `chars()`, so multi-byte
        // UTF-8 stays intact in the reported symbol — not split mid-byte
        // and stringified as "?".
        let err = parse("x > 0 ∧ y > 0", &[("x", 0), ("y", 1)]).unwrap_err();
        assert_eq!(
            err,
            BooleanParseError::UnknownSymbol {
                symbol: "∧".to_string()
            }
        );
    }
}
