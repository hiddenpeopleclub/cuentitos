//! Polymorphic value type for the variable/expression layer.
//!
//! [`Value`] holds a runtime payload; [`ValueKind`] is the type tag used at
//! parse time to drive type inference and reject mismatches before the
//! runtime ever sees the program. Both enums are designed to grow:
//! `Float(f64)` and `String(String)` slot in additively without reshaping
//! any consumer that already type-tests with `match`.

/// A runtime value carried by an expression literal, variable cell, or
/// evaluation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    // Future variants: Float(f64), String(String).
}

impl Value {
    /// Type tag for this value.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Integer(_) => ValueKind::Integer,
            Value::Boolean(_) => ValueKind::Boolean,
        }
    }

    /// Extract the integer payload if this value is an `Integer`, else `None`.
    /// Used at sites that the parser has already proven to be integer-typed —
    /// callers prefer an explicit `match` to make the type-test visible.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            Value::Boolean(_) => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
        }
    }
}

/// Type tag for [`Value`]. Used by the parse-time type inferrer to compare
/// expected vs. found types without inspecting payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Integer,
    Boolean,
    // Future variants: Float, String.
}

impl ValueKind {
    /// True when this kind supports `+ - * /`.
    #[must_use]
    pub fn is_numeric(self) -> bool {
        match self {
            ValueKind::Integer => true,
            ValueKind::Boolean => false,
        }
    }

    /// True when this kind supports `< <= > >=` ordering comparisons.
    #[must_use]
    pub fn is_ordered(self) -> bool {
        match self {
            ValueKind::Integer => true,
            ValueKind::Boolean => false,
        }
    }

    /// The source-syntax keyword that declares this kind (`int`, `bool`).
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
        }
    }
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Integer => write!(f, "integer"),
            ValueKind::Boolean => write!(f, "boolean"),
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
    }

    #[test]
    fn boolean_value_kind_displays_as_word() {
        assert_eq!(format!("{}", ValueKind::Boolean), "boolean");
    }
}
