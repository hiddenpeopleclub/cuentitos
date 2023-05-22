use crate::Variable;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Modifier {
  pub variable: Variable,
  pub new_value: String,
}
