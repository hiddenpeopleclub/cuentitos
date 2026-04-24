/// What type a declared variable has, together with its declared default.
///
/// Kept separate from [`VariableValue`] so that adding new runtime types
/// (e.g. `Bool`, `Float`, `String`) only requires extending two enums — no
/// storage migration for either the database or the runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableKind {
    /// 64-bit signed integer with the given default.
    Int(i64),
    // Future variants: Bool(bool), Float(f64), String(String).
}

impl VariableKind {
    /// Produce the initial [`VariableValue`] for this kind.
    pub fn initial_value(&self) -> VariableValue {
        match self {
            VariableKind::Int(n) => VariableValue::Int(*n),
        }
    }
}

/// The *current* value of a declared variable at runtime.
///
/// Mirrors [`VariableKind`] variant-for-variant. Equality with another
/// `VariableValue` only holds when both the variant *and* the payload match;
/// use [`VariableValue::same_kind`] for variant-only comparisons (needed by
/// the typed setter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableValue {
    Int(i64),
}

impl VariableValue {
    /// Extract the integer payload if this value is an `Int`, else `None`.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            VariableValue::Int(n) => Some(*n),
        }
    }

    /// True when `self` and `other` share the same variant (regardless of
    /// payload). Used by setters to reject type-mismatched writes.
    pub fn same_kind(&self, other: &VariableValue) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl std::fmt::Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::Int(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub kind: VariableKind,
}

impl Variable {
    pub fn new<S: Into<String>>(name: S, kind: VariableKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }

    /// Convenience constructor for integer variables.
    pub fn new_int<S: Into<String>>(name: S, default: i64) -> Self {
        Self::new(name, VariableKind::Int(default))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_int_builds_variable() {
        let v = Variable::new_int("an_integer", 42);
        assert_eq!(v.name, "an_integer");
        assert_eq!(v.kind, VariableKind::Int(42));
    }

    #[test]
    fn kind_initial_value_matches_variant() {
        assert_eq!(VariableKind::Int(7).initial_value(), VariableValue::Int(7));
    }

    #[test]
    fn value_as_int_extracts_payload() {
        assert_eq!(VariableValue::Int(3).as_int(), Some(3));
    }

    #[test]
    fn same_kind_ignores_payload() {
        assert!(VariableValue::Int(1).same_kind(&VariableValue::Int(999)));
    }

    #[test]
    fn value_display_formats_payload() {
        assert_eq!(format!("{}", VariableValue::Int(-5)), "-5");
    }
}
