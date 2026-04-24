#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub default_value: i64,
}

impl Variable {
    pub fn new<S: Into<String>>(name: S, default_value: i64) -> Self {
        Self {
            name: name.into(),
            default_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_new() {
        let v = Variable::new("an_integer", 42);
        assert_eq!(v.name, "an_integer");
        assert_eq!(v.default_value, 42);
    }
}
