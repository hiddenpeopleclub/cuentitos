pub struct Condition{
  pub operator: ComparisonOperator,
}

pub enum ComparisonOperator {
  Equals,
  HigherThan,
  LessThan,
  Depends,
  MutEx,
}