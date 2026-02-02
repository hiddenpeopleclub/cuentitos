use crate::StringId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableType {
    Integer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl ComparisonOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    Integer(i64),
}

impl VariableValue {
    pub fn to_display_string(&self) -> String {
        match self {
            VariableValue::Integer(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition {
    pub name: StringId,
    pub variable_type: VariableType,
    pub default_value: VariableValue,
}

impl VariableDefinition {
    pub fn new(name: StringId, variable_type: VariableType, default_value: VariableValue) -> Self {
        Self {
            name,
            variable_type,
            default_value,
        }
    }
}
