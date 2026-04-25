use crate::value::{Value, ValueKind};

/// What type a declared variable has, together with its declared default.
///
/// `kind` is the static [`ValueKind`] used by parse-time inference; `default`
/// is the [`Value`] handed to the runtime as the initial cell value. Adding a
/// new type means: extend [`Value`]/[`ValueKind`], add a constructor here,
/// and dispatch on the new variant where needed — no storage migration for
/// either the database or the runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub kind: ValueKind,
    pub default: Value,
}

impl Variable {
    /// Construct a variable from a name and a declared default. The variable's
    /// [`ValueKind`] is taken from the default.
    pub fn new<S: Into<String>>(name: S, default: Value) -> Self {
        Self {
            name: name.into(),
            kind: default.kind(),
            default,
        }
    }

    /// Convenience constructor for integer variables.
    pub fn new_integer<S: Into<String>>(name: S, default: i64) -> Self {
        Self::new(name, Value::Integer(default))
    }

    /// Produce the runtime value cell for this variable's declared default.
    #[must_use]
    pub fn initial_value(&self) -> Value {
        self.default.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_integer_builds_variable() {
        let v = Variable::new_integer("an_integer", 42);
        assert_eq!(v.name, "an_integer");
        assert_eq!(v.kind, ValueKind::Integer);
        assert_eq!(v.default, Value::Integer(42));
    }

    #[test]
    fn initial_value_clones_default() {
        let v = Variable::new_integer("x", 7);
        assert_eq!(v.initial_value(), Value::Integer(7));
    }
}
