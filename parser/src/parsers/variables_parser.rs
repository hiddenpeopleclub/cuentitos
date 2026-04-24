use cuentitos_common::{Database, Variable};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ParseError;

/// Outcome of parsing the variables block.
#[derive(Debug)]
pub struct VariablesBlockResult {
    /// Number of source lines consumed (including the opening and closing `---`).
    pub consumed_lines: usize,
}

/// Parse a `--- variables` block starting at `start_line_index` (0-based index
/// into `lines`). The caller must have already verified that
/// `lines[start_line_index].trim() == "--- variables"`.
///
/// On success, declared variables are appended to `database.variables` and the
/// number of consumed source lines is returned.
pub fn parse_variables_block(
    lines: &[&str],
    start_line_index: usize,
    database: &mut Database,
    file_path: &Option<PathBuf>,
) -> Result<VariablesBlockResult, ParseError> {
    let opening_line_number = start_line_index + 1;

    // Find the closing `---` line.
    let mut closing_line_index: Option<usize> = None;
    for (offset, line) in lines.iter().enumerate().skip(start_line_index + 1) {
        if line.trim() == "---" {
            closing_line_index = Some(offset);
            break;
        }
    }

    let closing_line_index = match closing_line_index {
        Some(i) => i,
        None => {
            return Err(ParseError::VariablesBlock {
                message: "Unterminated '--- variables' block: missing closing '---'.".to_string(),
                file: file_path.clone(),
                line: opening_line_number,
            });
        }
    };

    // First pass: collect all names that look like they will be declared in this
    // block, so that during evaluation we can distinguish between
    // "forward reference" and "truly undefined" references.
    let future_names = collect_future_names(lines, start_line_index + 1, closing_line_index);

    // Second pass: fully parse and evaluate each declaration in order.
    let mut declared_lines: HashMap<String, usize> = HashMap::new();
    let mut declared_values: HashMap<String, i64> = HashMap::new();

    for offset in (start_line_index + 1)..closing_line_index {
        let raw_line = lines[offset];
        let line_number = offset + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            return Err(ParseError::VariablesBlock {
                message: format!(
                    "Malformed variable declaration: '{}'. Declarations must not be indented.",
                    trimmed
                ),
                file: file_path.clone(),
                line: line_number,
            });
        }

        let rest = match trimmed.strip_prefix("int ") {
            Some(rest) => rest.trim_start(),
            None => {
                if trimmed == "int" {
                    return Err(ParseError::VariablesBlock {
                        message: "Malformed variable declaration: missing variable name."
                            .to_string(),
                        file: file_path.clone(),
                        line: line_number,
                    });
                }
                return Err(ParseError::VariablesBlock {
                    message: format!(
                        "Malformed variable declaration: '{}'. Expected 'int <name> [= <expr>]'.",
                        trimmed
                    ),
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
            return Err(ParseError::VariablesBlock {
                message: "Malformed variable declaration: missing variable name.".to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }

        if !is_valid_identifier(name) {
            return Err(ParseError::VariablesBlock {
                message: format!(
                    "Invalid variable name: '{}'. Variable names must start with a letter or underscore.",
                    name
                ),
                file: file_path.clone(),
                line: line_number,
            });
        }

        if let Some(&previous_line) = declared_lines.get(name) {
            return Err(ParseError::VariablesBlock {
                message: format!(
                    "Duplicate variable name: '{}' already declared. Previously declared at line {}.",
                    name, previous_line
                ),
                file: file_path.clone(),
                line: line_number,
            });
        }

        let value = if let Some(expr) = default_expr {
            if expr.is_empty() {
                return Err(ParseError::VariablesBlock {
                    message: format!("Malformed default expression: '{}'.", expr),
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            match evaluate_expression_internal(expr, &declared_values) {
                Ok(value) => value,
                Err(EvalError::Malformed) => {
                    return Err(ParseError::VariablesBlock {
                        message: format!("Malformed default expression: '{}'.", expr),
                        file: file_path.clone(),
                        line: line_number,
                    });
                }
                Err(EvalError::UndefinedVariable { referenced }) => {
                    if future_names.contains(&referenced) {
                        return Err(ParseError::VariablesBlock {
                            message: format!(
                                "Forward reference: variable '{}' referenced before declaration.",
                                referenced
                            ),
                            file: file_path.clone(),
                            line: line_number,
                        });
                    } else {
                        return Err(ParseError::VariablesBlock {
                            message: format!("Undefined variable: '{}'.", referenced),
                            file: file_path.clone(),
                            line: line_number,
                        });
                    }
                }
                Err(EvalError::DivisionByZero) => {
                    return Err(ParseError::VariablesBlock {
                        message: format!(
                            "Division by zero in default expression for '{}'.",
                            name
                        ),
                        file: file_path.clone(),
                        line: line_number,
                    });
                }
                Err(EvalError::Overflow) => {
                    return Err(ParseError::VariablesBlock {
                        message: format!(
                            "Integer overflow in default expression for '{}'.",
                            name
                        ),
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
        database.add_variable(Variable::new(name, value));
    }

    Ok(VariablesBlockResult {
        consumed_lines: closing_line_index - start_line_index + 1,
    })
}

/// Scan lines [start, end) (exclusive end) for identifiers that follow `int `,
/// collecting them into a set. Used to distinguish forward references from
/// truly undefined references.
fn collect_future_names(lines: &[&str], start: usize, end: usize) -> HashSet<String> {
    let mut names = HashSet::new();
    for offset in start..end {
        let trimmed = lines[offset].trim();
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
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---------------------------------------------------------------------------
// Expression evaluator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum EvalError {
    Malformed,
    UndefinedVariable { referenced: String },
    DivisionByZero,
    Overflow,
}

/// Evaluate an integer expression. Exposed via `evaluate_expression` for tests.
pub fn evaluate_expression(
    expression: &str,
    known_variables: &HashMap<String, i64>,
) -> Result<i64, EvalErrorPublic> {
    evaluate_expression_internal(expression, known_variables).map_err(EvalErrorPublic::from_private)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalErrorPublic {
    Malformed,
    UndefinedVariable { name: String },
    DivisionByZero,
    Overflow,
}

impl EvalErrorPublic {
    fn from_private(err: EvalError) -> Self {
        match err {
            EvalError::Malformed => EvalErrorPublic::Malformed,
            EvalError::UndefinedVariable { referenced } => {
                EvalErrorPublic::UndefinedVariable { name: referenced }
            }
            EvalError::DivisionByZero => EvalErrorPublic::DivisionByZero,
            EvalError::Overflow => EvalErrorPublic::Overflow,
        }
    }
}

fn evaluate_expression_internal(
    expression: &str,
    known_variables: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    let tokens = tokenize(expression).map_err(|_| EvalError::Malformed)?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Int(i64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ()> {
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
                let n: i64 = buf.parse().map_err(|_| ())?;
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
            _ => return Err(()),
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

fn parse_primary(
    tokens: &[Token],
    pos: &mut usize,
    vars: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    match tokens.get(*pos).cloned() {
        Some(Token::Int(n)) => {
            *pos += 1;
            Ok(n)
        }
        Some(Token::Ident(name)) => {
            *pos += 1;
            match vars.get(&name) {
                Some(&value) => Ok(value),
                None => Err(EvalError::UndefinedVariable { referenced: name }),
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

    #[test]
    fn eval_literal() {
        assert_eq!(evaluate_expression("42", &HashMap::new()).unwrap(), 42);
    }

    #[test]
    fn eval_negative_literal() {
        assert_eq!(evaluate_expression("-5", &HashMap::new()).unwrap(), -5);
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
            EvalErrorPublic::DivisionByZero
        );
    }

    #[test]
    fn eval_overflow() {
        assert_eq!(
            evaluate_expression("9223372036854775807 + 1", &HashMap::new()).unwrap_err(),
            EvalErrorPublic::Overflow
        );
    }

    #[test]
    fn eval_malformed_dangling() {
        assert_eq!(
            evaluate_expression("5 +", &HashMap::new()).unwrap_err(),
            EvalErrorPublic::Malformed
        );
    }

    #[test]
    fn eval_malformed_extra_paren() {
        assert_eq!(
            evaluate_expression("(1 + 2", &HashMap::new()).unwrap_err(),
            EvalErrorPublic::Malformed
        );
    }

    #[test]
    fn eval_undefined_reference() {
        let err = evaluate_expression("unknown", &HashMap::new()).unwrap_err();
        assert_eq!(
            err,
            EvalErrorPublic::UndefinedVariable {
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
        let res = parse_variables_block(&lines, 0, &mut db, &None).unwrap();
        assert_eq!(res.consumed_lines, 3);
        assert_eq!(db.variables.len(), 1);
        assert_eq!(db.variables[0].name, "five");
        assert_eq!(db.variables[0].default_value, 5);
    }

    #[test]
    fn parse_block_no_default_defaults_to_zero() {
        let script = "--- variables\nint a\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        parse_variables_block(&lines, 0, &mut db, &None).unwrap();
        assert_eq!(db.variables[0].default_value, 0);
    }

    #[test]
    fn parse_block_reference_earlier() {
        let script = "--- variables\nint a = 3\nint b = a + 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        parse_variables_block(&lines, 0, &mut db, &None).unwrap();
        assert_eq!(db.variables.len(), 2);
        assert_eq!(db.variables[1].default_value, 4);
    }

    #[test]
    fn parse_block_unterminated() {
        let script = "--- variables\nint a = 1\n";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 1);
                assert!(message.contains("Unterminated"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_duplicate_name() {
        let script = "--- variables\nint a\nint b = 1\nint a = 2\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 4);
                assert!(message.contains("Duplicate variable name"));
                assert!(message.contains("Previously declared at line 2"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_forward_reference() {
        let script = "--- variables\nint a = b\nint b = 5\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 2);
                assert!(message.contains("Forward reference"));
                assert!(message.contains("'b'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_undefined_reference() {
        let script = "--- variables\nint a = unknown\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 2);
                assert!(message.contains("Undefined variable"));
                assert!(message.contains("'unknown'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_invalid_identifier() {
        let script = "--- variables\nint 2foo = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 2);
                assert!(message.contains("Invalid variable name"));
                assert!(message.contains("'2foo'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_division_by_zero() {
        let script = "--- variables\nint a = 10 / 0\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 2);
                assert!(message.contains("Division by zero"));
                assert!(message.contains("'a'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_overflow_through_variable() {
        let script = "--- variables\nint big = 9223372036854775807\nint boom = big + 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 3);
                assert!(message.contains("Integer overflow"));
                assert!(message.contains("'boom'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }

    #[test]
    fn parse_block_malformed_expression() {
        let script = "--- variables\nint a = 5 +\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let err = parse_variables_block(&lines, 0, &mut db, &None).unwrap_err();
        match err {
            ParseError::VariablesBlock { message, line, .. } => {
                assert_eq!(line, 2);
                assert!(message.contains("Malformed default expression"));
                assert!(message.contains("'5 +'"));
            }
            _ => panic!("expected VariablesBlock error"),
        }
    }
}
