use cuentitos_common::{Database, Variable};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ParseError;

/// Outcome of parsing a `--- variables` block.
///
/// `consumed_lines` always spans the block from its opening `--- variables`
/// through the line containing the closing `---` (inclusive). When the closing
/// `---` is missing, the outcome reports the whole remaining file as consumed
/// so the caller knows there's nothing left to parse outside the block.
///
/// `errors` collects any per-declaration errors found inside the block. An
/// empty vector means the block parsed cleanly.
#[derive(Debug)]
pub struct VariablesBlockOutcome {
    pub consumed_lines: usize,
    pub errors: Vec<ParseError>,
}

/// Parse a `--- variables` block starting at `start_line_index` (0-based index
/// into `lines`). The caller must have already verified that
/// `lines[start_line_index].trim() == "--- variables"`.
///
/// On clean parses, declared variables are appended to `database.variables` in
/// declaration order. If an error is encountered on a declaration, parsing of
/// this block stops but the outcome still reports the full block span (when
/// the closing `---` was found) so the main parser can resume after it.
pub fn parse_variables_block(
    lines: &[&str],
    start_line_index: usize,
    database: &mut Database,
    file_path: &Option<PathBuf>,
) -> VariablesBlockOutcome {
    let opening_line_number = start_line_index + 1;

    // Find the closing `---` line. When absent, there is no block boundary, so
    // we report the whole rest of the file as consumed.
    let closing_line_index = lines
        .iter()
        .enumerate()
        .skip(start_line_index + 1)
        .find(|(_, line)| line.trim() == "---")
        .map(|(i, _)| i);

    let closing_line_index = match closing_line_index {
        Some(i) => i,
        None => {
            return VariablesBlockOutcome {
                consumed_lines: lines.len() - start_line_index,
                errors: vec![ParseError::UnterminatedVariablesBlock {
                    file: file_path.clone(),
                    line: opening_line_number,
                }],
            };
        }
    };

    // The block span spans [start_line_index, closing_line_index] inclusive.
    let block_span = closing_line_index - start_line_index + 1;

    // First pass: collect names that look like declarations in this block so
    // we can distinguish forward references from truly undefined references.
    // This set tolerates duplicates silently; duplicate *declaration* detection
    // happens in the main pass via `declared_lines`.
    let future_names = collect_future_names(lines, start_line_index + 1, closing_line_index);

    // Second pass: parse and evaluate each declaration in order.
    let mut declared_lines: HashMap<String, usize> = HashMap::new();
    let mut declared_values: HashMap<String, i64> = HashMap::new();

    for (offset, raw_line) in lines
        .iter()
        .copied()
        .enumerate()
        .take(closing_line_index)
        .skip(start_line_index + 1)
    {
        let line_number = offset + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Err(error) = parse_one_declaration(
            raw_line,
            trimmed,
            line_number,
            file_path,
            &future_names,
            &mut declared_lines,
            &mut declared_values,
            database,
        ) {
            return VariablesBlockOutcome {
                consumed_lines: block_span,
                errors: vec![error],
            };
        }
    }

    VariablesBlockOutcome {
        consumed_lines: block_span,
        errors: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn parse_one_declaration(
    raw_line: &str,
    trimmed: &str,
    line_number: usize,
    file_path: &Option<PathBuf>,
    future_names: &HashSet<String>,
    declared_lines: &mut HashMap<String, usize>,
    declared_values: &mut HashMap<String, i64>,
    database: &mut Database,
) -> Result<(), ParseError> {
    if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
        return Err(ParseError::IndentedVariableDeclaration {
            content: trimmed.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    let rest = match trimmed.strip_prefix("int ") {
        Some(rest) => rest.trim_start(),
        None => {
            if trimmed == "int" {
                return Err(ParseError::MissingVariableName {
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            return Err(ParseError::MalformedVariableDeclaration {
                content: trimmed.to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }
    };

    let (name, default_expr) = if let Some(eq_idx) = rest.find('=') {
        let name = rest[..eq_idx].trim();
        let expr = rest[eq_idx + 1..].trim();
        (name, Some(expr))
    } else {
        (rest.trim(), None)
    };

    if name.is_empty() {
        return Err(ParseError::MissingVariableName {
            file: file_path.clone(),
            line: line_number,
        });
    }

    if !is_valid_identifier(name) {
        return Err(ParseError::InvalidVariableName {
            name: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    if is_reserved_keyword(name) {
        return Err(ParseError::ReservedKeyword {
            name: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    if let Some(&previous_line) = declared_lines.get(name) {
        return Err(ParseError::DuplicateVariable {
            name: name.to_string(),
            previous_line,
            file: file_path.clone(),
            line: line_number,
        });
    }

    let value = if let Some(expr) = default_expr {
        match evaluate_expression_internal(expr, declared_values) {
            Ok(value) => value,
            Err(EvalError::Malformed) => {
                return Err(ParseError::MalformedDefaultExpression {
                    expr: expr.to_string(),
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            Err(EvalError::UndefinedVariable {
                name: referenced_name,
            }) => {
                if referenced_name == name {
                    return Err(ParseError::SelfReferenceInDefault {
                        name: referenced_name,
                        file: file_path.clone(),
                        line: line_number,
                    });
                }
                if future_names.contains(&referenced_name) {
                    return Err(ParseError::ForwardVariableReference {
                        name: referenced_name,
                        file: file_path.clone(),
                        line: line_number,
                    });
                }
                return Err(ParseError::UndefinedVariableReference {
                    name: referenced_name,
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            Err(EvalError::DivisionByZero) => {
                return Err(ParseError::DivisionByZero {
                    variable: name.to_string(),
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            Err(EvalError::Overflow) => {
                return Err(ParseError::IntegerOverflow {
                    variable: name.to_string(),
                    file: file_path.clone(),
                    line: line_number,
                });
            }
        }
    } else {
        0
    };

    declared_lines.insert(name.to_string(), line_number);
    declared_values.insert(name.to_string(), value);
    database.add_variable(Variable::new_integer(name, value));
    Ok(())
}

/// Scan lines `[start, end)` (exclusive end) for identifiers that follow
/// `int `, collecting them into a set. Duplicate identifiers are merged
/// silently here; duplicate-declaration detection lives in the main pass.
fn collect_future_names(lines: &[&str], start: usize, end: usize) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in &lines[start..end] {
        let trimmed = line.trim();
        let rest = match trimmed.strip_prefix("int ") {
            Some(r) => r.trim_start(),
            None => continue,
        };
        let name = if let Some(eq_idx) = rest.find('=') {
            rest[..eq_idx].trim()
        } else {
            rest.trim()
        };
        if is_valid_identifier(name) {
            names.insert(name.to_string());
        }
    }
    names
}

pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().expect("non-empty checked above");
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// True for lowercase logical-operator keywords that cannot be used as
/// variable names. Uppercase variants (`AND`/`OR`/`NOT`) are not reserved
/// — they parse as ordinary identifiers.
pub fn is_reserved_keyword(name: &str) -> bool {
    matches!(name, "and" | "or" | "not")
}

// ---------------------------------------------------------------------------
// Expression evaluator
// ---------------------------------------------------------------------------

/// Errors that can surface while evaluating a default expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    Malformed,
    UndefinedVariable { name: String },
    DivisionByZero,
    Overflow,
}

/// Evaluate an integer expression against a map of already-known variables.
pub fn evaluate_expression(
    expression: &str,
    known_variables: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    evaluate_expression_internal(expression, known_variables)
}

fn evaluate_expression_internal(
    expression: &str,
    known_variables: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    let tokens = tokenize(expression).map_err(|TokenizeError| EvalError::Malformed)?;
    if tokens.is_empty() {
        return Err(EvalError::Malformed);
    }
    let mut pos = 0;
    let value = parse_expr(&tokens, &mut pos, known_variables)?;
    if pos != tokens.len() {
        return Err(EvalError::Malformed);
    }
    Ok(value)
}

/// Integer literals are tokenized as `u64` so that the magnitude of
/// `i64::MIN` (`9_223_372_036_854_775_808`) is representable. `parse_unary`
/// folds a leading `-` into the literal so the full negative range is usable.
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

fn parse_expr(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    parse_additive(tokens, pos, vars)
}

fn parse_additive(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    let mut left = parse_multiplicative(tokens, pos, vars)?;
    loop {
        match tokens.get(*pos) {
            Some(Token::Plus) => {
                *pos += 1;
                let right = parse_multiplicative(tokens, pos, vars)?;
                left = left.checked_add(right).ok_or(EvalError::Overflow)?;
            }
            Some(Token::Minus) => {
                *pos += 1;
                let right = parse_multiplicative(tokens, pos, vars)?;
                left = left.checked_sub(right).ok_or(EvalError::Overflow)?;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_multiplicative(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    let mut left = parse_unary(tokens, pos, vars)?;
    loop {
        match tokens.get(*pos) {
            Some(Token::Star) => {
                *pos += 1;
                let right = parse_unary(tokens, pos, vars)?;
                left = left.checked_mul(right).ok_or(EvalError::Overflow)?;
            }
            Some(Token::Slash) => {
                *pos += 1;
                let right = parse_unary(tokens, pos, vars)?;
                if right == 0 {
                    return Err(EvalError::DivisionByZero);
                }
                left = left.checked_div(right).ok_or(EvalError::Overflow)?;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_unary(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    match tokens.get(*pos) {
        Some(Token::Minus) => {
            *pos += 1;
            // Fold `-` directly into a following literal so that `i64::MIN`
            // (whose magnitude doesn't fit in `i64`) is representable.
            if let Some(Token::Int(n)) = tokens.get(*pos) {
                *pos += 1;
                return negate_u64_literal(*n);
            }
            let value = parse_unary(tokens, pos, vars)?;
            value.checked_neg().ok_or(EvalError::Overflow)
        }
        Some(Token::Plus) => {
            *pos += 1;
            parse_unary(tokens, pos, vars)
        }
        _ => parse_primary(tokens, pos, vars),
    }
}

/// Compute `-(n as i64)` without intermediate overflow. Handles the unique
/// case of `i64::MIN`, whose absolute value is `(i64::MAX as u64) + 1`.
fn negate_u64_literal(n: u64) -> Result<i64, EvalError> {
    const ABS_MIN: u64 = (i64::MAX as u64) + 1; // 9_223_372_036_854_775_808
    if n > ABS_MIN {
        return Err(EvalError::Overflow);
    }
    if n == ABS_MIN {
        return Ok(i64::MIN);
    }
    Ok(-(n as i64))
}

fn parse_primary(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    match tokens.get(*pos) {
        Some(Token::Int(n)) => {
            let n = *n;
            *pos += 1;
            if n > i64::MAX as u64 {
                return Err(EvalError::Overflow);
            }
            Ok(n as i64)
        }
        Some(Token::Ident(_)) => {
            // Defer cloning until we know we actually need the name (either as
            // an error payload or as a map key for a miss).
            let name = match &tokens[*pos] {
                Token::Ident(name) => name,
                _ => unreachable!(),
            };
            *pos += 1;
            match vars.get(name) {
                Some(&value) => Ok(value),
                None => Err(EvalError::UndefinedVariable { name: name.clone() }),
            }
        }
        Some(Token::LParen) => {
            *pos += 1;
            let value = parse_expr(tokens, pos, vars)?;
            match tokens.get(*pos) {
                Some(Token::RParen) => {
                    *pos += 1;
                    Ok(value)
                }
                _ => Err(EvalError::Malformed),
            }
        }
        _ => Err(EvalError::Malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars_from(pairs: &[(&str, i64)]) -> HashMap<String, i64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn expect_single_error(outcome: VariablesBlockOutcome) -> ParseError {
        assert_eq!(
            outcome.errors.len(),
            1,
            "expected exactly one error, got {:?}",
            outcome.errors
        );
        outcome.errors.into_iter().next().unwrap()
    }

    #[test]
    fn eval_literal() {
        assert_eq!(evaluate_expression("42", &HashMap::new()).unwrap(), 42);
    }

    #[test]
    fn eval_negative_literal() {
        assert_eq!(evaluate_expression("-5", &HashMap::new()).unwrap(), -5);
    }

    #[test]
    fn eval_i64_min_literal() {
        assert_eq!(
            evaluate_expression("-9223372036854775808", &HashMap::new()).unwrap(),
            i64::MIN
        );
    }

    #[test]
    fn eval_i64_min_minus_one_overflows() {
        assert_eq!(
            evaluate_expression("-9223372036854775809", &HashMap::new()).unwrap_err(),
            EvalError::Overflow
        );
    }

    #[test]
    fn eval_parens() {
        assert_eq!(
            evaluate_expression("((1 + 2) * (3 + 4))", &HashMap::new()).unwrap(),
            21
        );
    }

    #[test]
    fn eval_reference() {
        let vars = vars_from(&[("a", 3)]);
        assert_eq!(evaluate_expression("a + 2", &vars).unwrap(), 5);
    }

    #[test]
    fn eval_integer_division_truncates_toward_zero() {
        assert_eq!(evaluate_expression("10 / 3", &HashMap::new()).unwrap(), 3);
        assert_eq!(evaluate_expression("-10 / 3", &HashMap::new()).unwrap(), -3);
        assert_eq!(evaluate_expression("10 / -3", &HashMap::new()).unwrap(), -3);
    }

    #[test]
    fn eval_div_by_zero() {
        assert_eq!(
            evaluate_expression("10 / 0", &HashMap::new()).unwrap_err(),
            EvalError::DivisionByZero
        );
    }

    #[test]
    fn eval_overflow() {
        assert_eq!(
            evaluate_expression("9223372036854775807 + 1", &HashMap::new()).unwrap_err(),
            EvalError::Overflow
        );
    }

    #[test]
    fn eval_malformed_dangling() {
        assert_eq!(
            evaluate_expression("5 +", &HashMap::new()).unwrap_err(),
            EvalError::Malformed
        );
    }

    #[test]
    fn eval_malformed_extra_paren() {
        assert_eq!(
            evaluate_expression("(1 + 2", &HashMap::new()).unwrap_err(),
            EvalError::Malformed
        );
    }

    #[test]
    fn eval_undefined_reference() {
        let err = evaluate_expression("unknown", &HashMap::new()).unwrap_err();
        assert_eq!(
            err,
            EvalError::UndefinedVariable {
                name: "unknown".to_string()
            }
        );
    }

    #[test]
    fn identifier_rules() {
        assert!(is_valid_identifier("a"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("foo_bar_123"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("2foo"));
        assert!(!is_valid_identifier("foo bar"));
        assert!(!is_valid_identifier("foo-bar"));
    }

    #[test]
    fn parse_block_literal_default() {
        let script = "--- variables\nint five = 5\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(outcome.consumed_lines, 3);
        assert_eq!(db.variables.len(), 1);
        assert_eq!(db.variables[0].name, "five");
        assert_eq!(db.variables[0].kind(), cuentitos_common::ValueKind::Integer);
        assert_eq!(db.variables[0].default, cuentitos_common::Value::Integer(5));
    }

    #[test]
    fn parse_block_no_default_defaults_to_zero() {
        let script = "--- variables\nint a\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables[0].default, cuentitos_common::Value::Integer(0));
    }

    #[test]
    fn parse_block_reference_earlier() {
        let script = "--- variables\nint a = 3\nint b = a + 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables.len(), 2);
        assert_eq!(db.variables[1].default, cuentitos_common::Value::Integer(4));
    }

    #[test]
    fn parse_block_unterminated() {
        let script = "--- variables\nint a = 1\n";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::UnterminatedVariablesBlock { line, .. } => assert_eq!(line, 1),
            other => panic!("expected UnterminatedVariablesBlock, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_duplicate_name() {
        let script = "--- variables\nint a\nint b = 1\nint a = 2\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DuplicateVariable {
                name,
                previous_line,
                line,
                ..
            } => {
                assert_eq!(name, "a");
                assert_eq!(previous_line, 2);
                assert_eq!(line, 4);
            }
            other => panic!("expected DuplicateVariable, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_forward_reference() {
        let script = "--- variables\nint a = b\nint b = 5\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::ForwardVariableReference { name, line, .. } => {
                assert_eq!(name, "b");
                assert_eq!(line, 2);
            }
            other => panic!("expected ForwardVariableReference, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_self_reference_is_its_own_error() {
        let script = "--- variables\nint a = a\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::SelfReferenceInDefault { name, line, .. } => {
                assert_eq!(name, "a");
                assert_eq!(line, 2);
            }
            other => panic!("expected SelfReferenceInDefault, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_undefined_reference() {
        let script = "--- variables\nint a = unknown\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::UndefinedVariableReference { name, line, .. } => {
                assert_eq!(name, "unknown");
                assert_eq!(line, 2);
            }
            other => panic!("expected UndefinedVariableReference, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_invalid_identifier() {
        let script = "--- variables\nint 2foo = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::InvalidVariableName { name, line, .. } => {
                assert_eq!(name, "2foo");
                assert_eq!(line, 2);
            }
            other => panic!("expected InvalidVariableName, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_reserved_keyword_and() {
        let script = "--- variables\nint and = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::ReservedKeyword { name, line, .. } => {
                assert_eq!(name, "and");
                assert_eq!(line, 2);
            }
            other => panic!("expected ReservedKeyword, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_reserved_keyword_or() {
        let script = "--- variables\nint or = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::ReservedKeyword { name, .. } => assert_eq!(name, "or"),
            other => panic!("expected ReservedKeyword, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_reserved_keyword_not() {
        let script = "--- variables\nint not = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::ReservedKeyword { name, .. } => assert_eq!(name, "not"),
            other => panic!("expected ReservedKeyword, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_uppercase_logical_keywords_are_allowed() {
        // The reservation is for the lowercase tokens the boolean parser
        // recognizes; uppercase variants must remain ordinary identifiers.
        let script = "--- variables\nint AND = 1\nint OR = 2\nint NOT = 3\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
    }

    #[test]
    fn parse_block_division_by_zero() {
        let script = "--- variables\nint a = 10 / 0\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DivisionByZero { variable, line, .. } => {
                assert_eq!(variable, "a");
                assert_eq!(line, 2);
            }
            other => panic!("expected DivisionByZero, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_overflow_through_variable() {
        let script = "--- variables\nint big = 9223372036854775807\nint boom = big + 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::IntegerOverflow { variable, line, .. } => {
                assert_eq!(variable, "boom");
                assert_eq!(line, 3);
            }
            other => panic!("expected IntegerOverflow, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_malformed_expression() {
        let script = "--- variables\nint a = 5 +\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::MalformedDefaultExpression { expr, line, .. } => {
                assert_eq!(expr, "5 +");
                assert_eq!(line, 2);
            }
            other => panic!("expected MalformedDefaultExpression, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_indented_declaration() {
        let script = "--- variables\n  int a = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::IndentedVariableDeclaration { content, line, .. } => {
                assert_eq!(content, "int a = 1");
                assert_eq!(line, 2);
            }
            other => panic!("expected IndentedVariableDeclaration, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_missing_variable_name() {
        let script = "--- variables\nint\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::MissingVariableName { line, .. } => assert_eq!(line, 2),
            other => panic!("expected MissingVariableName, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_i64_min_default() {
        let script = "--- variables\nint a = -9223372036854775808\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(
            db.variables[0].default,
            cuentitos_common::Value::Integer(i64::MIN)
        );
    }
}
