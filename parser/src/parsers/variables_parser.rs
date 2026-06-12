use cuentitos_common::{
    evaluate, Database, EvaluationError, Value, ValueKind, Variable, VariableId,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::expression::{parse_expression, ParseExpressionError};
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

    // Second pass: parse and evaluate each declaration in order. `declared`
    // maps each already-declared name to its folded default value, carrying
    // both the value and (via `Value::kind`) its type so later defaults can
    // reference earlier variables and type-check across kinds.
    let mut declared_lines: HashMap<String, usize> = HashMap::new();
    let mut declared: HashMap<String, Value> = HashMap::new();

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
            &mut declared,
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
    declared: &mut HashMap<String, Value>,
    database: &mut Database,
) -> Result<(), ParseError> {
    if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
        return Err(ParseError::IndentedVariableDeclaration {
            content: trimmed.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    // Dispatch on the declared type keyword. Each recognized keyword consumes
    // its `<keyword> ` prefix; a bare keyword with no name is a missing-name
    // error, and anything else is a malformed declaration.
    let (kind, rest) = match declaration_kind(trimmed) {
        Some((kind, rest)) => (kind, rest),
        None => {
            if trimmed == "int"
                || trimmed == "bool"
                || trimmed == "float"
                || trimmed == "string"
                || trimmed == "enum"
            {
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

    // Enum declarations have a completely different RHS grammar (a
    // comma-separated value list, not an arithmetic default). Dispatch before
    // the shared name/duplicate/default-fold logic so the enum path owns all
    // of its own diagnostics.
    if kind == ValueKind::Enum {
        return parse_enum_declaration(
            rest,
            line_number,
            file_path,
            declared_lines,
            declared,
            database,
        );
    }

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

    // Fold the default into a concrete `Value` of the declared kind. Each
    // kind owns its own default grammar: integers fold arithmetic; booleans
    // accept only `true`/`false` or a reference to an earlier bool.
    let value = match kind {
        ValueKind::Integer => integer_default_value(
            name,
            default_expr,
            declared,
            future_names,
            file_path,
            line_number,
        )?,
        ValueKind::Boolean => boolean_default_value(
            name,
            default_expr,
            declared,
            future_names,
            file_path,
            line_number,
        )?,
        ValueKind::Float => float_default_value(
            name,
            default_expr,
            declared,
            future_names,
            file_path,
            line_number,
        )?,
        ValueKind::String => string_default_value(
            name,
            default_expr,
            declared,
            future_names,
            file_path,
            line_number,
        )?,
        // Enum is handled above via the early-return path.
        ValueKind::Enum => unreachable!("enum handled before this match"),
    };

    declared_lines.insert(name.to_string(), line_number);
    declared.insert(name.to_string(), value.clone());
    database.add_variable(Variable::new(name, value));
    Ok(())
}

/// Parse an `enum <name> = <value1>, <value2>, ...` declaration (the `rest`
/// argument is everything after the `enum ` keyword, e.g. `mood = happy, sad`).
///
/// Validates the name (identifier, non-reserved, non-duplicate), then parses
/// and validates the comma-separated value list, and finally registers the
/// variable in the database with an `EnumUnset` initial value.
fn parse_enum_declaration(
    rest: &str,
    line_number: usize,
    file_path: &Option<PathBuf>,
    declared_lines: &mut HashMap<String, usize>,
    declared: &mut HashMap<String, Value>,
    database: &mut Database,
) -> Result<(), ParseError> {
    // Split on the first `=` to separate name from value list.
    let (name_raw, value_list_raw) = match rest.find('=') {
        Some(eq_idx) => (rest[..eq_idx].trim(), Some(rest[eq_idx + 1..].trim())),
        None => (rest.trim(), None),
    };

    let name = name_raw;

    if name.is_empty() {
        return Err(ParseError::MissingVariableName {
            file: file_path.clone(),
            line: line_number,
        });
    }

    // Validate name: for `enum mood happy, sad` (no `=`) the "name" would be
    // `mood happy, sad` — not a valid identifier, but the missing-equals error
    // is more informative. We detect the missing `=` and report it specifically.
    if value_list_raw.is_none() {
        // There was no `=`. The first word is the name; whatever follows is noise.
        let name_word = name.split_whitespace().next().unwrap_or(name);
        return Err(ParseError::EnumMissingEquals {
            name: name_word.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    // The name itself should be a valid identifier at this point.
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

    let raw_list = value_list_raw.unwrap();

    // An empty value list is an error.
    if raw_list.is_empty() {
        return Err(ParseError::EnumEmptyValueList {
            name: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        });
    }

    // Parse the comma-separated value list. Each entry must be a non-empty,
    // valid identifier that is not a reserved keyword. Duplicates within this
    // enum are also rejected (but two different enums sharing a value name is
    // fine — the duplicate check is per-enum, not global).
    let mut variants: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for raw_value in raw_list.split(',') {
        let value = raw_value.trim();
        if value.is_empty() {
            return Err(ParseError::EnumEmptyValue {
                name: name.to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }
        if !is_valid_identifier(value) {
            return Err(ParseError::EnumInvalidValue {
                value: value.to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }
        if is_reserved_keyword(value) {
            return Err(ParseError::EnumReservedKeywordValue {
                keyword: value.to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }
        if !seen.insert(value.to_string()) {
            return Err(ParseError::EnumDuplicateValue {
                value: value.to_string(),
                enum_name: name.to_string(),
                file: file_path.clone(),
                line: line_number,
            });
        }
        variants.push(value.to_string());
    }

    let initial_value = Value::EnumUnset {
        variants: variants.clone(),
    };

    declared_lines.insert(name.to_string(), line_number);
    // Store a sentinel in `declared` so later declarations can see this name.
    // We use the `EnumUnset` value directly; the type checks for other kinds
    // (integer, bool, float, string) already reject cross-kind references, so
    // storing `EnumUnset` here means any attempt to use an enum name in another
    // variable's default will hit the type-mismatch path naturally.
    declared.insert(name.to_string(), initial_value.clone());
    database.add_variable(Variable::new(name, initial_value));
    Ok(())
}

/// Recognize a declaration's leading type keyword. Returns the declared
/// [`ValueKind`] and the remainder of the line (the name and optional
/// default / value-list), with leading whitespace after the keyword stripped.
/// `None` means the line is not a recognized `<keyword> ...` declaration.
fn declaration_kind(trimmed: &str) -> Option<(ValueKind, &str)> {
    if let Some(rest) = trimmed.strip_prefix("int ") {
        Some((ValueKind::Integer, rest.trim_start()))
    } else if let Some(rest) = trimmed.strip_prefix("bool ") {
        Some((ValueKind::Boolean, rest.trim_start()))
    } else if let Some(rest) = trimmed.strip_prefix("float ") {
        Some((ValueKind::Float, rest.trim_start()))
    } else if let Some(rest) = trimmed.strip_prefix("string ") {
        Some((ValueKind::String, rest.trim_start()))
    } else if let Some(rest) = trimmed.strip_prefix("enum ") {
        Some((ValueKind::Enum, rest.trim_start()))
    } else {
        None
    }
}

/// Fold an integer variable's default into a [`Value::Integer`]. With no
/// default the value is `0`. Errors are mapped to the integer-specific
/// `ParseError` diagnostics (overflow, division-by-zero, forward/undefined
/// references) so the existing wording is preserved verbatim.
fn integer_default_value(
    name: &str,
    default_expr: Option<&str>,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
    file_path: &Option<PathBuf>,
    line_number: usize,
) -> Result<Value, ParseError> {
    let expr = match default_expr {
        Some(expr) => expr,
        None => return Ok(Value::Integer(0)),
    };

    // The arithmetic folder only sees integer-typed variables; project the
    // mixed-kind `declared` map down to its integer entries. A reference to a
    // bool variable therefore reads as undefined to the folder, which the
    // dispatch below resolves: an already-declared non-integer is a type
    // mismatch (handled first), so it never falls through to the forward /
    // undefined arms.
    let integer_view: HashMap<String, i64> = declared
        .iter()
        .filter_map(|(name, value)| value.as_integer().map(|n| (name.clone(), n)))
        .collect();

    match evaluate_expression(expr, &integer_view) {
        Ok(value) => Ok(Value::Integer(value)),
        Err(EvalError::Malformed) => Err(ParseError::MalformedDefaultExpression {
            expr: expr.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(EvalError::UndefinedVariable {
            name: referenced_name,
        }) => {
            // A name the folder couldn't resolve may still be declared earlier
            // with a non-integer kind (e.g. `int x = flag` where `flag` is an
            // earlier bool). That is a type mismatch, not a missing reference —
            // report it before the self / forward / undefined arms so the
            // diagnostic is symmetric with the bool folder's wrong-kind path.
            if let Some(value) = declared.get(&referenced_name) {
                return Err(ParseError::DefaultTypeMismatch {
                    variable: name.to_string(),
                    expected: ValueKind::Integer,
                    found_token: referenced_name,
                    found: value.kind(),
                    file: file_path.clone(),
                    line: line_number,
                });
            }
            if referenced_name == name {
                Err(ParseError::SelfReferenceInDefault {
                    name: referenced_name,
                    file: file_path.clone(),
                    line: line_number,
                })
            } else if future_names.contains(&referenced_name) {
                Err(ParseError::ForwardVariableReference {
                    name: referenced_name,
                    file: file_path.clone(),
                    line: line_number,
                })
            } else {
                Err(ParseError::UndefinedVariableReference {
                    name: referenced_name,
                    file: file_path.clone(),
                    line: line_number,
                })
            }
        }
        Err(EvalError::LiteralOverflow { literal }) => Err(ParseError::DefaultLiteralOverflow {
            variable: name.to_string(),
            literal,
            file: file_path.clone(),
            line: line_number,
        }),
        Err(EvalError::DivisionByZero) => Err(ParseError::DivisionByZero {
            variable: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(EvalError::Overflow) => Err(ParseError::IntegerOverflow {
            variable: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
    }
}

/// Fold a boolean variable's default into a [`Value::Boolean`]. With no
/// default the value is `false`. Errors are mapped to the relevant
/// `ParseError` diagnostics.
fn boolean_default_value(
    name: &str,
    default_expr: Option<&str>,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
    file_path: &Option<PathBuf>,
    line_number: usize,
) -> Result<Value, ParseError> {
    let expr = match default_expr {
        Some(expr) => expr,
        None => return Ok(Value::Boolean(false)),
    };

    match evaluate_bool_default(expr, name, declared, future_names) {
        Ok(value) => Ok(Value::Boolean(value)),
        Err(BoolDefaultError::LogicalOperator) => Err(ParseError::LogicalOperatorInDefault {
            file: file_path.clone(),
            line: line_number,
        }),
        Err(BoolDefaultError::TypeMismatch { token, found }) => {
            Err(ParseError::DefaultTypeMismatch {
                variable: name.to_string(),
                expected: ValueKind::Boolean,
                found_token: token,
                found,
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(BoolDefaultError::SelfReference) => Err(ParseError::SelfReferenceInDefault {
            name: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(BoolDefaultError::ForwardReference {
            name: referenced_name,
        }) => Err(ParseError::ForwardVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
        Err(BoolDefaultError::UndefinedReference {
            name: referenced_name,
        }) => Err(ParseError::UndefinedVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
    }
}

/// Errors that can surface while folding a boolean default. Kept as a private
/// enum so [`boolean_default_value`] owns the mapping to `ParseError`,
/// mirroring how [`EvalError`] decouples the integer folder from its
/// diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BoolDefaultError {
    /// A logical operator (`and`/`or`/`not`) appeared in the default.
    LogicalOperator,
    /// The default is a non-bool value used where a bool is required.
    /// `token` is the offending text; `found` is the kind it actually has.
    TypeMismatch { token: String, found: ValueKind },
    /// The default references the variable being declared.
    SelfReference,
    /// The default references a variable declared later in the same block.
    ForwardReference { name: String },
    /// The default references a name that is never declared.
    UndefinedReference { name: String },
}

/// Fold a boolean default expression against the variables declared earlier
/// in the same block.
///
/// A bool default is intentionally narrow: it is `true`, `false`, or a
/// reference to an earlier bool variable. Logical operators belong in `req`
/// and are rejected outright. Anything else is a type mismatch — and since
/// the language's only other value kind today is `int`, any non-bool,
/// non-logical default is reported as `int`.
fn evaluate_bool_default(
    expression: &str,
    variable_name: &str,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
) -> Result<bool, BoolDefaultError> {
    // Reject logical operators as whole whitespace-delimited tokens so
    // identifiers like `or_flag` and the (unreserved) uppercase `AND` are
    // unaffected. This must precede the literal check so `true or false`
    // routes here rather than matching the leading `true`.
    if expression
        .split_whitespace()
        .any(|token| matches!(token, "and" | "or" | "not"))
    {
        return Err(BoolDefaultError::LogicalOperator);
    }

    let trimmed = expression.trim();
    match trimmed {
        "true" => return Ok(true),
        "false" => return Ok(false),
        _ => {}
    }

    if is_valid_identifier(trimmed) {
        if trimmed == variable_name {
            return Err(BoolDefaultError::SelfReference);
        }
        if let Some(value) = declared.get(trimmed) {
            return match value {
                Value::Boolean(b) => Ok(*b),
                other => Err(BoolDefaultError::TypeMismatch {
                    token: trimmed.to_string(),
                    found: other.kind(),
                }),
            };
        }
        if future_names.contains(trimmed) {
            return Err(BoolDefaultError::ForwardReference {
                name: trimmed.to_string(),
            });
        }
        return Err(BoolDefaultError::UndefinedReference {
            name: trimmed.to_string(),
        });
    }

    // Not a bool literal and not a bare identifier: today that can only be an
    // integer-typed expression (e.g. `1`). Name the whole token and report it
    // as `int`. When float/string defaults land, refine the inferred kind.
    Err(BoolDefaultError::TypeMismatch {
        token: trimmed.to_string(),
        found: ValueKind::Integer,
    })
}

/// Fold a float variable's default into a [`Value::Float`]. With no default
/// the value is `0.0`. Errors are mapped to the float-specific `ParseError`
/// diagnostics (invalid literal, division-by-zero, overflow-to-infinity,
/// cross-type / forward / undefined references).
fn float_default_value(
    name: &str,
    default_expr: Option<&str>,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
    file_path: &Option<PathBuf>,
    line_number: usize,
) -> Result<Value, ParseError> {
    let expr = match default_expr {
        Some(expr) => expr,
        None => return Ok(Value::Float(0.0)),
    };

    match evaluate_float_default(expr, name, declared, future_names) {
        Ok(value) => Ok(Value::Float(value)),
        Err(FloatDefaultError::InvalidLiteral { literal }) => {
            Err(ParseError::InvalidFloatLiteral {
                literal,
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(FloatDefaultError::Malformed) => Err(ParseError::MalformedDefaultExpression {
            expr: expr.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(FloatDefaultError::DivisionByZero) => Err(ParseError::DivisionByZero {
            variable: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(FloatDefaultError::Overflow) => Err(ParseError::FloatOverflow {
            variable: name.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(FloatDefaultError::TypeMismatch { token, found }) => {
            Err(ParseError::FloatDefaultTypeMismatch {
                variable: name.to_string(),
                found_token: token,
                found,
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(FloatDefaultError::ForwardReference {
            name: referenced_name,
        }) => Err(ParseError::ForwardVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
        Err(FloatDefaultError::UndefinedReference {
            name: referenced_name,
        }) => Err(ParseError::UndefinedVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
    }
}

/// Errors that can surface while folding a float default. Kept private so
/// [`float_default_value`] owns the mapping to `ParseError`, mirroring how
/// [`BoolDefaultError`] decouples the bool folder from its diagnostics.
#[derive(Debug, Clone, PartialEq)]
enum FloatDefaultError {
    /// A numeric token wasn't a valid `<digits>.<digits>` float literal
    /// (e.g. `.5`, `1.`, `1e3`, or a bare integer). Carries the offending
    /// text so the diagnostic can name it.
    InvalidLiteral { literal: String },
    /// The expression didn't parse (dangling operator, unbalanced paren,
    /// unexpected symbol, or excessive nesting depth).
    Malformed,
    /// A `/ 0.0` was constant-folded. Defaults evaluate at parse time, so
    /// this is an error rather than an IEEE infinity.
    DivisionByZero,
    /// A finite operation produced a non-finite result (magnitude beyond the
    /// largest finite `f64`). Reported rather than storing an infinity.
    Overflow,
    /// The default referenced a non-float value where a float was required.
    /// `token` is the offending text; `found` is the kind it actually has.
    TypeMismatch { token: String, found: ValueKind },
    /// The default references a variable declared later in the same block.
    /// (A self-reference lands here too — the name is not yet in scope on its
    /// own declaration line.)
    ForwardReference { name: String },
    /// The default references a name that is never declared.
    UndefinedReference { name: String },
}

/// The float default sublanguage's token alphabet. Float literals are
/// pre-folded to `f64` during tokenization so the parser only ever sees
/// finished numbers.
#[derive(Debug, Clone, PartialEq)]
enum FloatToken {
    Float(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
}

/// Constant-fold a float default expression against the variables declared
/// earlier in the same block.
///
/// A float default is `+ - * /`, parentheses, and unary minus over float
/// literals (`<digits>.<digits>`) and references to earlier float variables.
/// Division is IEEE (no truncation); division by zero and overflow-to-infinity
/// are parse-time errors because defaults fold here rather than at runtime.
fn evaluate_float_default(
    expression: &str,
    variable_name: &str,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
) -> Result<f64, FloatDefaultError> {
    let tokens = tokenize_float_expression(expression)?;
    if tokens.is_empty() {
        return Err(FloatDefaultError::Malformed);
    }
    let mut folder = FloatFolder {
        tokens: &tokens,
        position: 0,
        variable_name,
        declared,
        future_names,
        depth: 0,
    };
    let value = folder.parse_additive()?;
    if folder.position != tokens.len() {
        return Err(FloatDefaultError::Malformed);
    }
    Ok(value)
}

/// Lex a float default expression. Numeric tokens are validated as
/// `<digits>.<digits>` and folded to `f64`; anything that starts like a number
/// but isn't (`.5`, `1.`, `1e3`, a bare integer) is captured whole and
/// reported as an invalid literal.
fn tokenize_float_expression(input: &str) -> Result<Vec<FloatToken>, FloatDefaultError> {
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
                tokens.push(FloatToken::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(FloatToken::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(FloatToken::Star);
            }
            '/' => {
                chars.next();
                tokens.push(FloatToken::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(FloatToken::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(FloatToken::RightParen);
            }
            // A number-like run starts with a digit or a `.`. Consume the
            // maximal contiguous run of alphanumerics and dots so a malformed
            // literal (`1e3`, `1.5.5`, `.5`, `1.`) is captured whole for the
            // diagnostic, then validate it as `<digits>.<digits>`.
            c if c.is_ascii_digit() || c == '.' => {
                let mut lexeme = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '.' {
                        lexeme.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(FloatToken::Float(parse_float_literal(&lexeme)?));
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
                tokens.push(FloatToken::Identifier(buf));
            }
            _ => return Err(FloatDefaultError::Malformed),
        }
    }
    Ok(tokens)
}

/// Validate a numeric lexeme as a `<digits>.<digits>` float literal and parse
/// it to `f64`. Anything else (leading/trailing dot, exponent, extra dots, a
/// bare integer) is an [`FloatDefaultError::InvalidLiteral`].
fn parse_float_literal(lexeme: &str) -> Result<f64, FloatDefaultError> {
    let invalid = || FloatDefaultError::InvalidLiteral {
        literal: lexeme.to_string(),
    };
    let (integer_part, fractional_part) = lexeme.split_once('.').ok_or_else(invalid)?;
    let is_digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    if !is_digits(integer_part) || !is_digits(fractional_part) {
        return Err(invalid());
    }
    // A well-formed `<digits>.<digits>` lexeme always parses, but a magnitude
    // beyond the largest finite `f64` parses to `Ok(±infinity)` rather than
    // `Err`. Reject that as an overflow rather than storing the infinity,
    // mirroring how a constant-folded product overflow is caught — a single
    // huge literal must not slip through where `1e200 * 1e200` is rejected.
    let value = lexeme.parse::<f64>().map_err(|_| invalid())?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FloatDefaultError::Overflow)
    }
}

/// Recursive-descent folder over a pre-lexed float default expression. Folds
/// straight to `f64` (no AST) against the float-typed variables declared
/// earlier in the block.
struct FloatFolder<'a> {
    tokens: &'a [FloatToken],
    position: usize,
    variable_name: &'a str,
    declared: &'a HashMap<String, Value>,
    future_names: &'a HashSet<String>,
    depth: usize,
}

impl FloatFolder<'_> {
    fn peek(&self) -> Option<&FloatToken> {
        self.tokens.get(self.position)
    }

    fn enter_recursion(&mut self) -> Result<(), FloatDefaultError> {
        self.depth += 1;
        if self.depth > crate::boolean_expression::MAX_EXPRESSION_DEPTH {
            return Err(FloatDefaultError::Malformed);
        }
        Ok(())
    }

    fn leave_recursion(&mut self) {
        self.depth -= 1;
    }

    fn parse_additive(&mut self) -> Result<f64, FloatDefaultError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let subtract = match self.peek() {
                Some(FloatToken::Plus) => false,
                Some(FloatToken::Minus) => true,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_multiplicative()?;
            left = finite(if subtract { left - right } else { left + right })?;
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<f64, FloatDefaultError> {
        let mut left = self.parse_unary()?;
        loop {
            let divide = match self.peek() {
                Some(FloatToken::Star) => false,
                Some(FloatToken::Slash) => true,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_unary()?;
            if divide {
                if right == 0.0 {
                    return Err(FloatDefaultError::DivisionByZero);
                }
                left = finite(left / right)?;
            } else {
                left = finite(left * right)?;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<f64, FloatDefaultError> {
        match self.peek() {
            Some(FloatToken::Minus) => {
                self.position += 1;
                self.enter_recursion()?;
                let inner = self.parse_unary()?;
                self.leave_recursion();
                Ok(-inner)
            }
            Some(FloatToken::Plus) => {
                self.position += 1;
                self.enter_recursion()?;
                let inner = self.parse_unary()?;
                self.leave_recursion();
                Ok(inner)
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<f64, FloatDefaultError> {
        match self.peek() {
            Some(FloatToken::Float(value)) => {
                let value = *value;
                self.position += 1;
                Ok(value)
            }
            Some(FloatToken::Identifier(name)) => {
                let name = name.clone();
                self.position += 1;
                self.resolve(&name)
            }
            Some(FloatToken::LeftParen) => {
                self.position += 1;
                self.enter_recursion()?;
                let inner = self.parse_additive()?;
                self.leave_recursion();
                match self.peek() {
                    Some(FloatToken::RightParen) => {
                        self.position += 1;
                        Ok(inner)
                    }
                    _ => Err(FloatDefaultError::Malformed),
                }
            }
            _ => Err(FloatDefaultError::Malformed),
        }
    }

    /// Resolve an identifier to the float value of an earlier declaration.
    /// A reference to an earlier non-float variable is a type mismatch; a name
    /// declared later in the block (including the variable's own name) is a
    /// forward reference; anything else is undefined.
    fn resolve(&self, name: &str) -> Result<f64, FloatDefaultError> {
        if let Some(value) = self.declared.get(name) {
            return match value.as_float() {
                Some(folded) => Ok(folded),
                None => Err(FloatDefaultError::TypeMismatch {
                    token: name.to_string(),
                    found: value.kind(),
                }),
            };
        }
        if name == self.variable_name || self.future_names.contains(name) {
            return Err(FloatDefaultError::ForwardReference {
                name: name.to_string(),
            });
        }
        Err(FloatDefaultError::UndefinedReference {
            name: name.to_string(),
        })
    }
}

/// Reject a non-finite fold result (overflow to ±infinity). NaN cannot arise
/// here: operands are always finite (literals are validated finite and earlier
/// variables already folded without overflow) and division-by-zero is caught
/// before the division runs.
fn finite(value: f64) -> Result<f64, FloatDefaultError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FloatDefaultError::Overflow)
    }
}

/// Fold a string variable's default into a [`Value::String`]. With no default
/// the value is the empty string. Errors are mapped to the relevant
/// `ParseError` diagnostics (unterminated literal, invalid escape, malformed
/// expression, cross-type / forward / undefined references).
fn string_default_value(
    name: &str,
    default_expr: Option<&str>,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
    file_path: &Option<PathBuf>,
    line_number: usize,
) -> Result<Value, ParseError> {
    let expr = match default_expr {
        Some(expr) => expr,
        None => return Ok(Value::String(String::new())),
    };

    match evaluate_string_default(expr, name, declared, future_names) {
        Ok(value) => Ok(Value::String(value)),
        Err(StringDefaultError::UnterminatedLiteral) => {
            Err(ParseError::UnterminatedStringLiteral {
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(StringDefaultError::InvalidEscape { sequence }) => {
            Err(ParseError::InvalidEscapeSequence {
                sequence,
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(StringDefaultError::Malformed) => Err(ParseError::MalformedDefaultExpression {
            expr: expr.to_string(),
            file: file_path.clone(),
            line: line_number,
        }),
        Err(StringDefaultError::TypeMismatch { token, found }) => {
            Err(ParseError::DefaultTypeMismatch {
                variable: name.to_string(),
                expected: ValueKind::String,
                found_token: token,
                found,
                file: file_path.clone(),
                line: line_number,
            })
        }
        Err(StringDefaultError::ForwardReference {
            name: referenced_name,
        }) => Err(ParseError::ForwardVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
        Err(StringDefaultError::UndefinedReference {
            name: referenced_name,
        }) => Err(ParseError::UndefinedVariableReference {
            name: referenced_name,
            file: file_path.clone(),
            line: line_number,
        }),
    }
}

/// Errors that can surface while folding a string default. Kept private so
/// [`string_default_value`] owns the mapping to `ParseError`, mirroring how
/// [`BoolDefaultError`] / [`FloatDefaultError`] decouple their folders from the
/// diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StringDefaultError {
    /// A double-quoted literal opened but never closed on the declaration line.
    UnterminatedLiteral,
    /// A backslash escape other than `\"`, `\n`, or `\\` appeared inside a
    /// literal. Carries the two-character offending sequence (e.g. `\q`).
    InvalidEscape { sequence: String },
    /// The default is neither a single string literal nor a bare reference —
    /// e.g. attempted concatenation (`"a" + "b"`) or any other unparseable
    /// expression. The caller echoes the original text.
    Malformed,
    /// The default references a non-string value where a string was required.
    /// `token` is the offending text; `found` is the kind it actually has.
    TypeMismatch { token: String, found: ValueKind },
    /// The default references a variable declared later in the same block.
    /// (A self-reference lands here too — the name is not yet in scope on its
    /// own declaration line.)
    ForwardReference { name: String },
    /// The default references a name that is never declared.
    UndefinedReference { name: String },
}

/// Resolve a string default against the variables declared earlier in the same
/// block.
///
/// A string default is intentionally narrow: it is a single double-quoted
/// literal (with `\"`, `\n`, `\\` escapes) or a bare identifier referencing an
/// earlier string variable. Concatenation and every other compound expression
/// are unsupported and surface as malformed.
fn evaluate_string_default(
    expression: &str,
    variable_name: &str,
    declared: &HashMap<String, Value>,
    future_names: &HashSet<String>,
) -> Result<String, StringDefaultError> {
    let trimmed = expression.trim();

    if trimmed.starts_with('"') {
        return parse_string_literal(trimmed);
    }

    if is_valid_identifier(trimmed) {
        if let Some(value) = declared.get(trimmed) {
            return match value.as_string() {
                Some(folded) => Ok(folded.to_string()),
                None => Err(StringDefaultError::TypeMismatch {
                    token: trimmed.to_string(),
                    found: value.kind(),
                }),
            };
        }
        if trimmed == variable_name || future_names.contains(trimmed) {
            return Err(StringDefaultError::ForwardReference {
                name: trimmed.to_string(),
            });
        }
        return Err(StringDefaultError::UndefinedReference {
            name: trimmed.to_string(),
        });
    }

    Err(StringDefaultError::Malformed)
}

/// Parse a double-quoted string literal that occupies the whole `input` (the
/// caller has already trimmed it and verified the leading `"`).
///
/// Delegates to the shared [`crate::string_literal`] scanner so string
/// *defaults* and string `set` RHS literals honor identical escape and
/// termination rules, then re-maps the shared error into the default-specific
/// [`StringDefaultError`]: trailing characters after the closing quote make
/// the default malformed (e.g. an attempted concatenation).
fn parse_string_literal(input: &str) -> Result<String, StringDefaultError> {
    use crate::string_literal::StringLiteralError;
    crate::string_literal::parse_string_literal(input).map_err(|error| match error {
        StringLiteralError::Unterminated => StringDefaultError::UnterminatedLiteral,
        StringLiteralError::InvalidEscape { sequence } => {
            StringDefaultError::InvalidEscape { sequence }
        }
        StringLiteralError::TrailingCharacters => StringDefaultError::Malformed,
    })
}

/// Scan lines `[start, end)` (exclusive end) for identifiers that follow a
/// type keyword (`int `/`bool `/`float `/`string `/`enum `), collecting them
/// into a set regardless of declared type. Duplicate identifiers are merged
/// silently here; duplicate-declaration detection lives in the main pass.
fn collect_future_names(lines: &[&str], start: usize, end: usize) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in &lines[start..end] {
        let trimmed = line.trim();
        let rest = match declaration_kind(trimmed) {
            Some((_, rest)) => rest,
            None => continue,
        };
        // For all types, the variable name is before any `=`. For `enum` this
        // is correct too: `enum mood = happy, sad` → rest = `mood = happy, sad`
        // → name = `mood`.
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
//
// Variable defaults are constant-folded at parse time: each declaration is
// reduced to a concrete `i64` *here* so the engine never has to re-evaluate
// the default at runtime. The arithmetic grammar itself is the same one
// `set` and `req` use, so this module no longer carries its own parser —
// it delegates to the shared body via [`crate::expression::parse_expression`]
// (which drives [`crate::arithmetic::parse_arithmetic_expression`]) and
// then folds the resulting AST with the runtime evaluator
// [`cuentitos_common::evaluate`]. Keeping a single parser + evaluator
// across the three sites means an edge-case fix in one place (e.g. the
// recent recursion-depth cap) is automatically inherited here instead of
// drifting against a hand-rolled copy.
//
// The choice of "parse with the shared body, fold with the runtime
// evaluator" — rather than walking the AST locally — is deliberate:
// `cuentitos_common::evaluate` already implements the exact checked
// arithmetic the default folder needs (overflow on `+`/`-`/`*`, divide-by-
// zero), so reusing it guarantees parse-time and runtime see identical
// arithmetic semantics by construction.

/// Errors that can surface while evaluating a default expression. This is
/// a thin re-mapping of upstream parse-time ([`ParseExpressionError`]) and
/// fold-time ([`EvaluationError`]) errors into the shape the call site
/// wants to dispatch on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    Malformed,
    /// A single literal in the expression exceeded the `i64` range.
    /// Carries the offending literal text (including any leading sign,
    /// per the shared body's `negate_u64_literal`) so the caller can
    /// name it in a diagnostic — parallel to the `set`/`req` literal-
    /// overflow paths.
    LiteralOverflow {
        literal: String,
    },
    UndefinedVariable {
        name: String,
    },
    DivisionByZero,
    /// Binary arithmetic overflowed during fold (e.g. `i64::MAX + 1`).
    /// Distinct from `LiteralOverflow`: this case has no single offending
    /// literal to name, so the caller surfaces a generic overflow
    /// diagnostic.
    Overflow,
}

/// Constant-fold a default expression against the values declared earlier
/// in the same block.
pub fn evaluate_expression(
    expression: &str,
    known_variables: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    // The shared parser binds identifiers to `VariableId`s. Defaults are
    // tracked by name -> value, so we synthesize a private id space here:
    // each known name gets the position of its entry in `names`, and the
    // resolver hands those positions back to the parser. The fold step
    // then indexes `values` by the same position.
    let names: Vec<&str> = known_variables.keys().map(String::as_str).collect();
    let values: Vec<Value> = names
        .iter()
        .map(|name| Value::Integer(known_variables[*name]))
        .collect();
    let resolver =
        |query: &str| -> Option<VariableId> { names.iter().position(|known| *known == query) };

    let expression_ast = match parse_expression(expression, &resolver) {
        Ok(ast) => ast,
        Err(ParseExpressionError::Malformed) => return Err(EvalError::Malformed),
        Err(ParseExpressionError::Overflow { literal }) => {
            return Err(EvalError::LiteralOverflow { literal });
        }
        // A float literal can only appear in a float default, which is folded
        // by `evaluate_float_default`, not here — so an overflowing one never
        // reaches the integer-default path. Mirrors the `Value::Float(_)`
        // arm below, which rejects a finite float on the same grounds.
        Err(ParseExpressionError::FloatOverflow { .. }) => {
            unreachable!("integer-default fold never receives a float literal")
        }
        Err(ParseExpressionError::UndefinedVariable { name }) => {
            return Err(EvalError::UndefinedVariable { name });
        }
    };

    match evaluate(&expression_ast, &|id: VariableId| &values[id]) {
        Ok(folded) => match folded.into_owned() {
            Value::Integer(n) => Ok(n),
            // The resolver only exposes integer-typed variables and the shared
            // parser only produces integer arithmetic, so the fold never
            // yields a boolean, float, string, or enum. (Boolean and float
            // defaults bypass this folder entirely — see `evaluate_bool_default`
            // / `evaluate_float_default`.)
            Value::Boolean(_) => unreachable!("integer-default fold never yields a boolean"),
            Value::Float(_) => unreachable!("integer-default fold never yields a float"),
            Value::String(_) => unreachable!("integer-default fold never yields a string"),
            Value::EnumUnset { .. } | Value::Enum { .. } => {
                unreachable!("integer-default fold never yields an enum")
            }
        },
        Err(EvaluationError::DivisionByZero) => Err(EvalError::DivisionByZero),
        Err(EvaluationError::Overflow) => Err(EvalError::Overflow),
        // The integer-default fold only produces integer arithmetic, so a
        // float overflow can never arise here (float defaults use
        // `evaluate_float_default`).
        Err(EvaluationError::FloatOverflow) => {
            unreachable!("integer-default fold never yields a float overflow")
        }
        // This folder is the integer-default path: its resolver hands back
        // only `Value::Integer`s and the shared parser only produces integer-
        // shaped expressions, so the fold step cannot mismatch types. Boolean
        // defaults never reach here. If this path ever folds a mixed-kind
        // expression, the assertion fires and forces a deliberate fix.
        Err(EvaluationError::TypeMismatch { .. }) => {
            unreachable!("integer-default fold is integer-only")
        }
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
        // One below i64::MIN surfaces as a *literal* overflow, with the
        // signed text preserved so the caller can name the offending
        // literal — parallel to the set/req literal-overflow path.
        assert_eq!(
            evaluate_expression("-9223372036854775809", &HashMap::new()).unwrap_err(),
            EvalError::LiteralOverflow {
                literal: "-9223372036854775809".to_string(),
            }
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

    #[test]
    fn parse_block_one_below_i64_min_surfaces_literal_overflow() {
        // Locks in the diagnostic-parity contract: a default whose
        // single offending literal exceeds the integer range surfaces
        // `DefaultLiteralOverflow` with the literal text preserved,
        // parallel to `SetLiteralOverflow` / `RequirementLiteralOverflow`.
        let script = "--- variables\nint a = -9223372036854775809\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DefaultLiteralOverflow {
                variable,
                literal,
                line,
                ..
            } => {
                assert_eq!(variable, "a");
                assert_eq!(literal, "-9223372036854775809");
                assert_eq!(line, 2);
            }
            other => panic!("expected DefaultLiteralOverflow, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_bare_positive_literal_overflow() {
        // A bare positive literal larger than i64::MAX (but still inside
        // u64) also surfaces as `DefaultLiteralOverflow` with the
        // literal text preserved — same shape as the set/req sibling
        // diagnostics for `99999999999999999999`.
        let script = "--- variables\nint a = 99999999999999999999\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DefaultLiteralOverflow {
                variable,
                literal,
                line,
                ..
            } => {
                assert_eq!(variable, "a");
                assert_eq!(literal, "99999999999999999999");
                assert_eq!(line, 2);
            }
            other => panic!("expected DefaultLiteralOverflow, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_multiplication_overflow() {
        // Binary overflow at fold time (no single offending literal)
        // routes to the generic `IntegerOverflow` variant — distinct
        // from `DefaultLiteralOverflow`. The pre-existing
        // `parse_block_overflow_through_variable` test covers the
        // addition case; this one pins multiplication so the shared
        // body's `checked_mul` path is also locked in.
        let script = "--- variables\nint big = 4611686018427387904\nint boom = big * 3\n---";
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
    fn parse_block_deep_unary_minus_fails_cleanly() {
        // 200 leading `-`s would stack-overflow the parser without the
        // shared body's recursion cap. The cap surfaces as
        // `ExpressionTooDeep` inside the arithmetic body, which the set-
        // side mapper folds into `Malformed` — so the variables-default
        // path sees a `MalformedDefaultExpression`. Same fail-cleanly
        // contract as the set side; the test pins behavior, not message.
        let mut expr = String::new();
        for _ in 0..200 {
            expr.push('-');
        }
        expr.push('1');
        let script = format!("--- variables\nint a = {}\n---", expr);
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::MalformedDefaultExpression { line, .. } => {
                assert_eq!(line, 2);
            }
            other => panic!("expected MalformedDefaultExpression, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_deep_paren_nesting_fails_cleanly() {
        // 200 nested `(`s exercise the LParen recursion in the shared
        // arithmetic body. Same fail-cleanly contract as the unary-minus
        // chain above.
        let mut expr = String::new();
        for _ in 0..200 {
            expr.push('(');
        }
        expr.push('1');
        for _ in 0..200 {
            expr.push(')');
        }
        let script = format!("--- variables\nint a = {}\n---", expr);
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::MalformedDefaultExpression { line, .. } => {
                assert_eq!(line, 2);
            }
            other => panic!("expected MalformedDefaultExpression, got {:?}", other),
        }
    }

    // -- boolean declarations --------------------------------------------

    fn declared_from(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect()
    }

    #[test]
    fn eval_bool_literal_true_and_false() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("true", "b", &declared, &future),
            Ok(true)
        );
        assert_eq!(
            evaluate_bool_default("false", "b", &declared, &future),
            Ok(false)
        );
    }

    #[test]
    fn eval_bool_reference_to_earlier_bool() {
        let declared = declared_from(&[("source", Value::Boolean(true))]);
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("source", "mirror", &declared, &future),
            Ok(true)
        );
    }

    #[test]
    fn eval_bool_int_literal_is_type_mismatch() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("1", "b", &declared, &future),
            Err(BoolDefaultError::TypeMismatch {
                token: "1".to_string(),
                found: ValueKind::Integer,
            })
        );
    }

    #[test]
    fn eval_bool_reference_to_int_is_type_mismatch() {
        let declared = declared_from(&[("count", Value::Integer(3))]);
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("count", "b", &declared, &future),
            Err(BoolDefaultError::TypeMismatch {
                token: "count".to_string(),
                found: ValueKind::Integer,
            })
        );
    }

    #[test]
    fn eval_bool_logical_operators_are_rejected() {
        let declared = HashMap::new();
        let future = HashSet::new();
        for expr in ["not true", "true or false", "true and false"] {
            assert_eq!(
                evaluate_bool_default(expr, "b", &declared, &future),
                Err(BoolDefaultError::LogicalOperator),
                "expr: {expr}"
            );
        }
    }

    #[test]
    fn eval_bool_uppercase_logical_words_are_ordinary_identifiers() {
        // `AND`/`OR`/`NOT` are not reserved; as a lone reference they resolve
        // like any other identifier (here: undefined), never as an operator.
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("AND", "b", &declared, &future),
            Err(BoolDefaultError::UndefinedReference {
                name: "AND".to_string()
            })
        );
    }

    #[test]
    fn eval_bool_self_reference() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_bool_default("a", "a", &declared, &future),
            Err(BoolDefaultError::SelfReference)
        );
    }

    #[test]
    fn eval_bool_forward_reference() {
        let declared = HashMap::new();
        let future: HashSet<String> = std::iter::once("b".to_string()).collect();
        assert_eq!(
            evaluate_bool_default("b", "a", &declared, &future),
            Err(BoolDefaultError::ForwardReference {
                name: "b".to_string()
            })
        );
    }

    #[test]
    fn parse_block_bool_literal_defaults() {
        let script = "--- variables\nbool t = true\nbool f = false\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
        assert_eq!(db.variables[0].default, Value::Boolean(true));
        assert_eq!(db.variables[0].kind(), ValueKind::Boolean);
        assert_eq!(db.variables[1].default, Value::Boolean(false));
    }

    #[test]
    fn parse_block_bool_no_default_is_false() {
        let script = "--- variables\nbool a\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables[0].default, Value::Boolean(false));
    }

    #[test]
    fn parse_block_bool_reference_earlier() {
        let script = "--- variables\nbool source = true\nbool mirror = source\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables[1].default, Value::Boolean(true));
    }

    #[test]
    fn parse_block_bool_int_interleaved_preserves_order_and_kinds() {
        let script = "--- variables\nint a = 1\nbool b = true\nint c\nbool d\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        let kinds: Vec<_> = db
            .variables
            .iter()
            .map(|v| (v.name.as_str(), &v.default))
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("a", &Value::Integer(1)),
                ("b", &Value::Boolean(true)),
                ("c", &Value::Integer(0)),
                ("d", &Value::Boolean(false)),
            ]
        );
    }

    #[test]
    fn parse_block_bool_default_not_bool_literal() {
        let script = "--- variables\nbool b = 1\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DefaultTypeMismatch {
                variable,
                expected,
                found_token,
                found,
                line,
                ..
            } => {
                assert_eq!(variable, "b");
                assert_eq!(expected, ValueKind::Boolean);
                assert_eq!(found_token, "1");
                assert_eq!(found, ValueKind::Integer);
                assert_eq!(line, 2);
            }
            other => panic!("expected DefaultTypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_bool_default_references_non_bool() {
        let script = "--- variables\nint count = 3\nbool b = count\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DefaultTypeMismatch {
                variable,
                found_token,
                found,
                line,
                ..
            } => {
                assert_eq!(variable, "b");
                assert_eq!(found_token, "count");
                assert_eq!(found, ValueKind::Integer);
                assert_eq!(line, 3);
            }
            other => panic!("expected DefaultTypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_int_default_references_earlier_bool() {
        // The inverse of `parse_block_bool_default_references_non_bool`: an
        // `int` default referencing an earlier `bool` must report a type
        // mismatch — not a forward/undefined reference, since the bool is
        // declared on the previous line.
        let script = "--- variables\nbool flag = true\nint x = flag\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::DefaultTypeMismatch {
                variable,
                expected,
                found_token,
                found,
                line,
                ..
            } => {
                assert_eq!(variable, "x");
                assert_eq!(expected, ValueKind::Integer);
                assert_eq!(found_token, "flag");
                assert_eq!(found, ValueKind::Boolean);
                assert_eq!(line, 3);
            }
            other => panic!("expected DefaultTypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_bool_logical_operator_in_default() {
        let script = "--- variables\nbool b = true or false\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::LogicalOperatorInDefault { line, .. } => assert_eq!(line, 2),
            other => panic!("expected LogicalOperatorInDefault, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_bool_forward_reference() {
        let script = "--- variables\nbool a = b\nbool b = true\n---";
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
    fn parse_block_bool_reserved_keyword() {
        let script = "--- variables\nbool and = true\n---";
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
    fn parse_block_duplicate_name_across_types() {
        let script = "--- variables\nint x\nbool x = true\n---";
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
                assert_eq!(name, "x");
                assert_eq!(previous_line, 2);
                assert_eq!(line, 3);
            }
            other => panic!("expected DuplicateVariable, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_bool_missing_name() {
        let script = "--- variables\nbool\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::MissingVariableName { line, .. } => assert_eq!(line, 2),
            other => panic!("expected MissingVariableName, got {:?}", other),
        }
    }

    #[test]
    fn bool_default_type_mismatch_display_names_both_keywords() {
        let err = ParseError::DefaultTypeMismatch {
            variable: "b".to_string(),
            expected: ValueKind::Boolean,
            found_token: "1".to_string(),
            found: ValueKind::Integer,
            file: None,
            line: 2,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:2: ERROR: Type mismatch: default for bool 'b' must be a bool, but '1' is int."
        );
    }

    #[test]
    fn logical_operator_in_default_display() {
        let err = ParseError::LogicalOperatorInDefault {
            file: None,
            line: 2,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:2: ERROR: Logical operators (and/or/not) are not allowed in variable defaults; use 'req' for boolean expressions."
        );
    }

    #[test]
    fn default_literal_overflow_display_mirrors_set_and_req() {
        // Pins the exact wording so set/req/default literal-overflow
        // diagnostics differ only by which expression context they name.
        let err = ParseError::DefaultLiteralOverflow {
            variable: "a".to_string(),
            literal: "99999999999999999999".to_string(),
            file: None,
            line: 2,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:2: ERROR: Integer overflow in default expression for 'a': literal '99999999999999999999' exceeds the integer range."
        );
    }

    // -- float declarations ----------------------------------------------

    fn eval_float(expr: &str, declared: &[(&str, Value)]) -> Result<f64, FloatDefaultError> {
        let declared = declared_from(declared);
        let future = HashSet::new();
        evaluate_float_default(expr, "target", &declared, &future)
    }

    #[test]
    fn eval_float_literal() {
        assert_eq!(eval_float("10.5", &[]), Ok(10.5));
    }

    #[test]
    fn eval_float_arithmetic_and_ieee_division() {
        assert_eq!(eval_float("(1.0 + 2.0) * 2.0", &[]), Ok(6.0));
        // Division is IEEE — no truncation, unlike the integer folder.
        assert_eq!(eval_float("7.0 / 2.0", &[]), Ok(3.5));
    }

    #[test]
    fn eval_float_unary_minus_on_literal_and_paren() {
        assert_eq!(eval_float("-5.0", &[]), Ok(-5.0));
        assert_eq!(eval_float("-(2.5 + 3.0)", &[]), Ok(-5.5));
    }

    #[test]
    fn eval_float_reference_to_earlier_float() {
        assert_eq!(
            eval_float("source * 2.0", &[("source", Value::Float(10.5))]),
            Ok(21.0)
        );
    }

    #[test]
    fn eval_float_division_by_zero_is_parse_error() {
        assert_eq!(
            eval_float("10.0 / 0.0", &[]),
            Err(FloatDefaultError::DivisionByZero)
        );
    }

    #[test]
    fn eval_float_overflow_to_infinity_is_error() {
        let huge = format!("{}.0", "1".to_string() + &"0".repeat(200));
        let expr = format!("{huge} * {huge}");
        assert_eq!(eval_float(&expr, &[]), Err(FloatDefaultError::Overflow));
    }

    #[test]
    fn eval_float_single_literal_beyond_range_is_overflow_not_infinity() {
        // A lone literal beyond the largest finite `f64` parses to infinity;
        // reject it as an overflow rather than storing the infinity, just as
        // the folded product above is rejected. `1e320` is out of range.
        let literal = format!("1{}.0", "0".repeat(320));
        assert_eq!(eval_float(&literal, &[]), Err(FloatDefaultError::Overflow));
    }

    #[test]
    fn eval_float_negative_zero_is_preserved() {
        // `-0.0` and `0.0 * -1.0` both fold to a negatively-signed zero.
        assert!(eval_float("-0.0", &[]).unwrap().is_sign_negative());
        assert!(eval_float("0.0 * -1.0", &[]).unwrap().is_sign_negative());
        assert!(eval_float("0.0", &[]).unwrap().is_sign_positive());
    }

    #[test]
    fn eval_float_invalid_literals() {
        for literal in [".5", "1.", "1e3", "1.5.5", "5"] {
            assert_eq!(
                eval_float(literal, &[]),
                Err(FloatDefaultError::InvalidLiteral {
                    literal: literal.to_string()
                }),
                "literal: {literal}"
            );
        }
    }

    #[test]
    fn eval_float_malformed_dangling_operator() {
        assert_eq!(eval_float("5.0 +", &[]), Err(FloatDefaultError::Malformed));
        assert_eq!(
            eval_float("(1.0 + 2.0", &[]),
            Err(FloatDefaultError::Malformed)
        );
    }

    #[test]
    fn eval_float_cross_type_reference_is_type_mismatch() {
        assert_eq!(
            eval_float("count * 2.0", &[("count", Value::Integer(3))]),
            Err(FloatDefaultError::TypeMismatch {
                token: "count".to_string(),
                found: ValueKind::Integer,
            })
        );
    }

    #[test]
    fn eval_float_self_reference_reported_as_forward_reference() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_float_default("a", "a", &declared, &future),
            Err(FloatDefaultError::ForwardReference {
                name: "a".to_string()
            })
        );
    }

    #[test]
    fn eval_float_forward_and_undefined_references() {
        let declared = HashMap::new();
        let future: HashSet<String> = std::iter::once("later".to_string()).collect();
        assert_eq!(
            evaluate_float_default("later", "a", &declared, &future),
            Err(FloatDefaultError::ForwardReference {
                name: "later".to_string()
            })
        );
        assert_eq!(
            evaluate_float_default("missing", "a", &declared, &future),
            Err(FloatDefaultError::UndefinedReference {
                name: "missing".to_string()
            })
        );
    }

    #[test]
    fn parse_block_float_literal_default() {
        let script = "--- variables\nfloat starting_health = 10.5\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
        assert_eq!(db.variables[0].kind(), ValueKind::Float);
        assert_eq!(db.variables[0].default, Value::Float(10.5));
    }

    #[test]
    fn parse_block_float_no_default_is_zero() {
        let script = "--- variables\nfloat a\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables[0].default, Value::Float(0.0));
    }

    #[test]
    fn parse_block_float_int_interleaved_preserves_order_and_kinds() {
        let script = "--- variables\nint count = 3\nfloat ratio = 1.5\nint total = count + 4\nfloat half = ratio / 2.0\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
        let rows: Vec<_> = db
            .variables
            .iter()
            .map(|v| (v.name.as_str(), v.default.clone()))
            .collect();
        assert_eq!(
            rows,
            vec![
                ("count", Value::Integer(3)),
                ("ratio", Value::Float(1.5)),
                ("total", Value::Integer(7)),
                ("half", Value::Float(0.75)),
            ]
        );
    }

    #[test]
    fn parse_block_float_invalid_literal_surfaces_diagnostic() {
        let script = "--- variables\nfloat x = .5\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::InvalidFloatLiteral { literal, line, .. } => {
                assert_eq!(literal, ".5");
                assert_eq!(line, 2);
            }
            other => panic!("expected InvalidFloatLiteral, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_float_cross_type_default() {
        let script = "--- variables\nint count = 3\nfloat ratio = count * 2.0\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::FloatDefaultTypeMismatch {
                variable,
                found_token,
                found,
                line,
                ..
            } => {
                assert_eq!(variable, "ratio");
                assert_eq!(found_token, "count");
                assert_eq!(found, ValueKind::Integer);
                assert_eq!(line, 3);
            }
            other => panic!("expected FloatDefaultTypeMismatch, got {:?}", other),
        }
    }

    #[test]
    fn float_invalid_literal_display() {
        let err = ParseError::InvalidFloatLiteral {
            literal: "1e3".to_string(),
            file: None,
            line: 2,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:2: ERROR: Invalid float literal: '1e3'. Float literals must be written as <digits>.<digits> (e.g. '1.5')."
        );
    }

    #[test]
    fn float_default_type_mismatch_display() {
        let err = ParseError::FloatDefaultTypeMismatch {
            variable: "ratio".to_string(),
            found_token: "count".to_string(),
            found: ValueKind::Integer,
            file: None,
            line: 3,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:3: ERROR: Type mismatch: default for float ratio must be a float expression, but count is int."
        );
    }

    #[test]
    fn float_overflow_display() {
        let err = ParseError::FloatOverflow {
            variable: "boom".to_string(),
            file: None,
            line: 2,
        };
        assert_eq!(
            format!("{}", err),
            "<script>:2: ERROR: Float overflow in default expression for 'boom'."
        );
    }

    // -- string declarations ---------------------------------------------

    #[test]
    fn eval_string_literal_default() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"Aria\"", "name", &declared, &future),
            Ok("Aria".to_string())
        );
    }

    #[test]
    fn eval_string_escapes_unescape_to_value() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"a\\nb\"", "s", &declared, &future),
            Ok("a\nb".to_string())
        );
        assert_eq!(
            evaluate_string_default("\"She said \\\"hi\\\"\"", "s", &declared, &future),
            Ok("She said \"hi\"".to_string())
        );
        assert_eq!(
            evaluate_string_default("\"a\\\\b\"", "s", &declared, &future),
            Ok("a\\b".to_string())
        );
    }

    #[test]
    fn eval_string_empty_literal() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"\"", "s", &declared, &future),
            Ok(String::new())
        );
    }

    #[test]
    fn eval_string_reference_to_earlier_string() {
        let declared = declared_from(&[("hero", Value::String("Aria".to_string()))]);
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("hero", "echo", &declared, &future),
            Ok("Aria".to_string())
        );
    }

    #[test]
    fn eval_string_unterminated_literal() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"Aria", "name", &declared, &future),
            Err(StringDefaultError::UnterminatedLiteral)
        );
    }

    #[test]
    fn eval_string_invalid_escape() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"a\\qb\"", "name", &declared, &future),
            Err(StringDefaultError::InvalidEscape {
                sequence: "\\q".to_string(),
            })
        );
    }

    #[test]
    fn eval_string_concatenation_is_malformed() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("\"Hello, \" + \"world\"", "g", &declared, &future),
            Err(StringDefaultError::Malformed)
        );
    }

    #[test]
    fn eval_string_reference_to_non_string_is_type_mismatch() {
        let declared = declared_from(&[("count", Value::Integer(7))]);
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("count", "name", &declared, &future),
            Err(StringDefaultError::TypeMismatch {
                token: "count".to_string(),
                found: ValueKind::Integer,
            })
        );
    }

    #[test]
    fn eval_string_forward_reference() {
        let declared = HashMap::new();
        let future: HashSet<String> = ["b".to_string()].into_iter().collect();
        assert_eq!(
            evaluate_string_default("b", "a", &declared, &future),
            Err(StringDefaultError::ForwardReference {
                name: "b".to_string(),
            })
        );
    }

    #[test]
    fn eval_string_undefined_reference() {
        let declared = HashMap::new();
        let future = HashSet::new();
        assert_eq!(
            evaluate_string_default("unknown", "a", &declared, &future),
            Err(StringDefaultError::UndefinedReference {
                name: "unknown".to_string(),
            })
        );
    }

    #[test]
    fn parse_block_string_literal_default() {
        let script = "--- variables\nstring name = \"Aria\"\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty(), "errors: {:?}", outcome.errors);
        assert_eq!(db.variables[0].name, "name");
        assert_eq!(db.variables[0].kind(), ValueKind::String);
        assert_eq!(db.variables[0].default, Value::String("Aria".to_string()));
    }

    #[test]
    fn parse_block_string_no_default_is_empty() {
        let script = "--- variables\nstring name\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        assert!(outcome.errors.is_empty());
        assert_eq!(db.variables[0].default, Value::String(String::new()));
    }

    #[test]
    fn parse_block_string_unterminated_literal_display() {
        let script = "--- variables\nstring name = \"Aria\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::UnterminatedStringLiteral { line, .. } => assert_eq!(line, 2),
            other => panic!("expected UnterminatedStringLiteral, got {:?}", other),
        }
    }

    #[test]
    fn parse_block_string_invalid_escape_display() {
        let script = "--- variables\nstring name = \"a\\qb\"\n---";
        let lines: Vec<&str> = script.lines().collect();
        let mut db = Database::new();
        let outcome = parse_variables_block(&lines, 0, &mut db, &None);
        match expect_single_error(outcome) {
            ParseError::InvalidEscapeSequence { sequence, line, .. } => {
                assert_eq!(sequence, "\\q");
                assert_eq!(line, 2);
            }
            other => panic!("expected InvalidEscapeSequence, got {:?}", other),
        }
    }
}
