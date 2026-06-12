//! Polymorphic value type for the variable/expression layer.
//!
//! [`Value`] holds a runtime payload; [`ValueKind`] is the type tag used at
//! parse time to drive type inference and reject mismatches before the
//! runtime ever sees the program. Both enums are designed to grow:
//! `String(String)` slots in additively without reshaping any consumer that
//! already type-tests with `match`.
//!
//! Note [`Value`] is only `PartialEq`, not `Eq`: the `Float(f64)` payload has
//! no total equality (`NaN != NaN`), so the derive was narrowed. Nothing uses
//! a `Value` as a hash key, so this costs nothing.

/// A runtime value carried by an expression literal, variable cell, or
/// evaluation result.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    Float(f64),
    String(String),
}

impl Value {
    /// Type tag for this value.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Integer(_) => ValueKind::Integer,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
        }
    }

    /// Extract the integer payload if this value is an `Integer`, else `None`.
    /// Used at sites that the parser has already proven to be integer-typed —
    /// callers prefer an explicit `match` to make the type-test visible.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            Value::Boolean(_) | Value::Float(_) | Value::String(_) => None,
        }
    }

    /// Extract the float payload if this value is a `Float`, else `None`.
    /// Parallel to [`as_integer`](Self::as_integer): used where the parser has
    /// already proven a value is float-typed (e.g. projecting the mixed-kind
    /// `declared` map down to its float entries in the default folder).
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(_) | Value::Boolean(_) | Value::String(_) => None,
        }
    }

    /// Extract the string payload if this value is a `String`, else `None`.
    /// Parallel to [`as_integer`](Self::as_integer) and
    /// [`as_float`](Self::as_float): used where the parser has already proven a
    /// value is string-typed (e.g. resolving a default reference to an earlier
    /// string variable).
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::Integer(_) | Value::Boolean(_) | Value::Float(_) => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Float(x) => write!(f, "{}", format_float(*x)),
            // The raw string content, unquoted. The double-quoted, escaped form
            // used by the `?` debug command lives in [`format_string_literal`].
            Value::String(s) => write!(f, "{s}"),
        }
    }
}

/// Render a string value in cuentitos' canonical double-quoted textual form,
/// re-applying the three supported escapes.
///
/// The contract (locked by the `variables-string` compatibility tests):
/// - the value is wrapped in double quotes;
/// - a literal double quote renders as `\"`;
/// - a backslash renders as `\\`;
/// - a newline renders as the two-character escape `\n` so the output stays on
///   one line.
///
/// This is the inverse of the string-literal parser's escape handling: a value
/// produced from a literal round-trips back to the same source escapes.
#[must_use]
pub fn format_string_literal(value: &str) -> String {
    let mut rendered = String::with_capacity(value.len() + 2);
    rendered.push('"');
    for character in value.chars() {
        match character {
            '"' => rendered.push_str("\\\""),
            '\\' => rendered.push_str("\\\\"),
            '\n' => rendered.push_str("\\n"),
            other => rendered.push(other),
        }
    }
    rendered.push('"');
    rendered
}

/// Render a float in cuentitos' canonical textual form.
///
/// The contract (locked by the `variables-float` compatibility tests):
/// - shortest decimal that round-trips back to the same `f64` (Rust's default
///   `f64` `Display` already guarantees this);
/// - never scientific / exponent notation (Rust's `Display` never emits one);
/// - always at least one fractional digit, so an integral value like `2.0`
///   renders as `2.0` rather than `2` and stays visibly distinct from the
///   integer kind;
/// - the sign of zero is preserved, so IEEE `-0.0` renders as `-0.0`.
#[must_use]
pub fn format_float(value: f64) -> String {
    // `{}` gives the shortest round-tripping, non-scientific form and keeps
    // the sign bit of zero (`-0.0` -> "-0"). It only omits the decimal point
    // for integral magnitudes, so append `.0` when no point is present.
    let rendered = format!("{value}");
    if rendered.contains('.') {
        rendered
    } else {
        format!("{rendered}.0")
    }
}

/// Type tag for [`Value`]. Used by the parse-time type inferrer to compare
/// expected vs. found types without inspecting payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Integer,
    Boolean,
    Float,
    String,
}

impl ValueKind {
    /// True when this kind supports `+ - * /`.
    #[must_use]
    pub fn is_numeric(self) -> bool {
        match self {
            ValueKind::Integer | ValueKind::Float => true,
            ValueKind::Boolean | ValueKind::String => false,
        }
    }

    /// True when this kind supports `< <= > >=` ordering comparisons.
    #[must_use]
    pub fn is_ordered(self) -> bool {
        match self {
            ValueKind::Integer | ValueKind::Float => true,
            ValueKind::Boolean | ValueKind::String => false,
        }
    }

    /// The source-syntax keyword that declares this kind (`int`, `bool`,
    /// `float`).
    ///
    /// Distinct from [`Display`](std::fmt::Display), which renders the full
    /// English word (`integer`, `boolean`) used in `set`/`req` diagnostics.
    /// Default-declaration diagnostics name the keyword instead so they echo
    /// what the author actually typed.
    #[must_use]
    pub fn keyword(self) -> &'static str {
        match self {
            ValueKind::Integer => "int",
            ValueKind::Boolean => "bool",
            ValueKind::Float => "float",
            ValueKind::String => "string",
        }
    }
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Integer => write!(f, "integer"),
            ValueKind::Boolean => write!(f, "boolean"),
            ValueKind::Float => write!(f, "float"),
            ValueKind::String => write!(f, "string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_kind_is_numeric_and_ordered() {
        assert!(ValueKind::Integer.is_numeric());
        assert!(ValueKind::Integer.is_ordered());
    }

    #[test]
    fn integer_value_kind_round_trip() {
        assert_eq!(Value::Integer(42).kind(), ValueKind::Integer);
    }

    #[test]
    fn integer_value_extracts_payload() {
        assert_eq!(Value::Integer(7).as_integer(), Some(7));
    }

    #[test]
    fn integer_value_displays_as_decimal() {
        assert_eq!(format!("{}", Value::Integer(-12)), "-12");
    }

    #[test]
    fn integer_value_kind_displays_as_word() {
        assert_eq!(format!("{}", ValueKind::Integer), "integer");
    }

    #[test]
    fn boolean_value_kind_round_trip() {
        assert_eq!(Value::Boolean(true).kind(), ValueKind::Boolean);
        assert_eq!(Value::Boolean(false).kind(), ValueKind::Boolean);
    }

    #[test]
    fn boolean_value_is_not_integer() {
        assert_eq!(Value::Boolean(true).as_integer(), None);
    }

    #[test]
    fn boolean_value_displays_as_true_or_false() {
        assert_eq!(format!("{}", Value::Boolean(true)), "true");
        assert_eq!(format!("{}", Value::Boolean(false)), "false");
    }

    #[test]
    fn boolean_kind_is_neither_numeric_nor_ordered() {
        assert!(!ValueKind::Boolean.is_numeric());
        assert!(!ValueKind::Boolean.is_ordered());
    }

    #[test]
    fn value_kind_keyword_matches_source_syntax() {
        assert_eq!(ValueKind::Integer.keyword(), "int");
        assert_eq!(ValueKind::Boolean.keyword(), "bool");
        assert_eq!(ValueKind::Float.keyword(), "float");
    }

    #[test]
    fn float_value_kind_round_trip() {
        assert_eq!(Value::Float(1.5).kind(), ValueKind::Float);
        assert_eq!(Value::Float(2.0).as_float(), Some(2.0));
        assert_eq!(Value::Float(2.0).as_integer(), None);
        assert_eq!(Value::Integer(2).as_float(), None);
    }

    #[test]
    fn float_kind_is_numeric_and_ordered() {
        assert!(ValueKind::Float.is_numeric());
        assert!(ValueKind::Float.is_ordered());
        assert_eq!(format!("{}", ValueKind::Float), "float");
    }

    #[test]
    fn format_float_always_has_a_fractional_digit() {
        assert_eq!(format_float(2.0), "2.0");
        assert_eq!(format_float(30.0), "30.0");
        assert_eq!(format_float(10.5), "10.5");
        assert_eq!(format_float(0.0), "0.0");
    }

    #[test]
    fn format_float_preserves_negative_zero() {
        assert_eq!(format_float(-0.0), "-0.0");
        // Bind the operands so the product (a runtime-computed `-0.0`) isn't
        // const-folded away by the `neg_multiply` lint — the point is to
        // exercise that a multiplied negative zero still renders signed.
        let zero = 0.0_f64;
        let negative_one = -1.0_f64;
        assert_eq!(format_float(zero * negative_one), "-0.0");
    }

    #[test]
    fn format_float_uses_shortest_round_tripping_decimal() {
        assert_eq!(format_float(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(format_float(1.0 / 3.0), "0.3333333333333333");
        assert_eq!(format_float(1.0 / 8.0), "0.125");
    }

    #[test]
    fn format_float_never_uses_scientific_notation() {
        assert_eq!(format_float(0.000001), "0.000001");
        assert_eq!(format_float(1_000_000_000_000_000.0), "1000000000000000.0");
    }

    #[test]
    fn float_value_displays_via_format_float() {
        assert_eq!(format!("{}", Value::Float(2.0)), "2.0");
        assert_eq!(format!("{}", Value::Float(-0.0)), "-0.0");
    }

    #[test]
    fn boolean_value_kind_displays_as_word() {
        assert_eq!(format!("{}", ValueKind::Boolean), "boolean");
    }

    #[test]
    fn string_value_kind_round_trip() {
        assert_eq!(Value::String("hi".to_string()).kind(), ValueKind::String);
        assert_eq!(Value::String("hi".to_string()).as_string(), Some("hi"));
        assert_eq!(Value::String("hi".to_string()).as_integer(), None);
        assert_eq!(Value::String("hi".to_string()).as_float(), None);
        assert_eq!(Value::Integer(2).as_string(), None);
    }

    #[test]
    fn string_kind_is_neither_numeric_nor_ordered() {
        assert!(!ValueKind::String.is_numeric());
        assert!(!ValueKind::String.is_ordered());
    }

    #[test]
    fn string_value_kind_keyword_and_word() {
        assert_eq!(ValueKind::String.keyword(), "string");
        assert_eq!(format!("{}", ValueKind::String), "string");
    }

    #[test]
    fn string_value_displays_raw_content() {
        assert_eq!(format!("{}", Value::String("Aria".to_string())), "Aria");
    }

    #[test]
    fn format_string_literal_quotes_and_escapes() {
        assert_eq!(format_string_literal("Aria"), "\"Aria\"");
        assert_eq!(format_string_literal(""), "\"\"");
        // A literal newline renders as the two-character escape, staying on one line.
        assert_eq!(format_string_literal("a\nb"), "\"a\\nb\"");
        // A literal double quote and backslash re-escape.
        assert_eq!(
            format_string_literal("She said \"hi\""),
            "\"She said \\\"hi\\\"\""
        );
        assert_eq!(format_string_literal("a\\b"), "\"a\\\\b\"");
    }
}
