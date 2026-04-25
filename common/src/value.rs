//! Polymorphic value type for the variable/expression layer.
//!
//! [`Value`] holds a runtime payload; [`ValueKind`] is the type tag used at
//! parse time to drive type inference and reject mismatches before the
//! runtime ever sees the program. Both enums are designed to grow:
//! `Boolean(bool)`, `Float(f64)`, and `String(String)` slot in additively
//! without reshaping any consumer that already type-tests with `match`.

/// A runtime value carried by an expression literal, variable cell, or
/// evaluation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    // Future variants: Boolean(bool), Float(f64), String(String).
}

impl Value {
    /// Type tag for this value.
    #[must_use]
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Integer(_) => ValueKind::Integer,
        }
    }

    /// Extract the integer payload if this value is an `Integer`, else `None`.
    /// Used at sites that the parser has already proven to be integer-typed —
    /// callers prefer an explicit `match` to make the type-test visible.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{n}"),
        }
    }
}

/// Type tag for [`Value`]. Used by the parse-time type inferrer to compare
/// expected vs. found types without inspecting payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Integer,
    // Future variants: Boolean, Float, String.
}

impl ValueKind {
    /// True when this kind supports `+ - * /`.
    #[must_use]
    pub fn is_numeric(self) -> bool {
        match self {
            ValueKind::Integer => true,
        }
    }

    /// True when this kind supports `< <= > >=` ordering comparisons.
    #[must_use]
    pub fn is_ordered(self) -> bool {
        match self {
            ValueKind::Integer => true,
        }
    }
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Integer => write!(f, "integer"),
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
}
