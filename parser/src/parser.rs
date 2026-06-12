use crate::parsers::{
    go_to_section_and_back_parser::GoToSectionAndBackParser,
    go_to_section_parser::GoToSectionParser, line_parser::LineParser, option_parser::OptionParser,
    section_parser::SectionParser, FeatureParser, ParserContext,
};
use cuentitos_common::*;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Parser {
    last_block_at_level: Vec<BlockId>, // Stack to track last block at each level
    last_section_at_level: Vec<BlockId>, // Stack to track last section at each level (for section hierarchy)
    seen_non_option_by_parent: HashMap<BlockId, bool>, // Track if a parent has seen any non-option child
    file_path: Option<PathBuf>,
    line_parser: LineParser,
    section_parser: SectionParser,
    option_parser: OptionParser,
    go_to_section_and_back_parser: GoToSectionAndBackParser,
    go_to_section_parser: GoToSectionParser,
    // Track section names by parent_id -> (name -> first_line_number)
    section_names_by_parent: HashMap<Option<BlockId>, HashMap<String, usize>>,
    // Track section ids by parent_id -> (id -> first_line_number)
    section_ids_by_parent: HashMap<Option<BlockId>, HashMap<String, usize>>,
    // Track goto paths temporarily during parsing (BlockId -> path)
    goto_paths: HashMap<BlockId, String>,
    // Collect errors instead of returning immediately
    errors: Vec<ParseError>,
    // Collect warnings
    warnings: Vec<Warning>,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        file: Option<PathBuf>,
        line: usize,
    },
    UnexpectedEndOfFile {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidIndentation {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
    SectionWithoutTitle {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidSectionHierarchy {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
    DuplicateSectionName {
        name: String,
        parent_name: String,
        file: Option<PathBuf>,
        line: usize,
        previous_line: usize,
    },
    InvalidGoToSection {
        message: String,
        file: Option<PathBuf>,
        line: usize,
    },
    SectionNotFound {
        path: String,
        file: Option<PathBuf>,
        line: usize,
    },
    NavigationAboveRoot {
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidSectionName {
        message: String,
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    EmptySection {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    OptionsWithoutParent {
        file: Option<PathBuf>,
        line: usize,
    },
    UnterminatedVariablesBlock {
        file: Option<PathBuf>,
        line: usize,
    },
    DuplicateVariable {
        name: String,
        previous_line: usize,
        file: Option<PathBuf>,
        line: usize,
    },
    MalformedDefaultExpression {
        expr: String,
        file: Option<PathBuf>,
        line: usize,
    },
    DivisionByZero {
        variable: String,
        file: Option<PathBuf>,
        line: usize,
    },
    IntegerOverflow {
        variable: String,
        file: Option<PathBuf>,
        line: usize,
    },
    InvalidVariableName {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    UndefinedVariableReference {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    ForwardVariableReference {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    SelfReferenceInDefault {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    IndentedVariableDeclaration {
        content: String,
        file: Option<PathBuf>,
        line: usize,
    },
    MissingVariableName {
        file: Option<PathBuf>,
        line: usize,
    },
    MalformedVariableDeclaration {
        content: String,
        file: Option<PathBuf>,
        line: usize,
    },
    MalformedSetExpression {
        expression: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A `set` assigned a value of one kind to a variable of a different
    /// declared kind, or a compound assignment targeted a non-numeric
    /// variable.
    SetTypeMismatch {
        variable: String,
        expected: cuentitos_common::ValueKind,
        found: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A `set` on a float variable had a non-float RHS. Parallel in shape to
    /// [`FloatDefaultTypeMismatch`](Self::FloatDefaultTypeMismatch): names the
    /// offending sub-expression (`found_token`) and its kind rather than just
    /// reporting two kinds, so the wording echoes what the author typed.
    FloatSetTypeMismatch {
        variable: String,
        found_token: String,
        found: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A compound `set` operator (`+= -= *= /=`) targeted a non-numeric
    /// variable.
    NonNumericAssignment {
        variable: String,
        kind: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` had a syntactically incomplete expression.
    MalformedRequirementExpression {
        expression: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` contained a symbol the tokenizer didn't recognize as part
    /// of any grammar token. This includes both unsupported comparison
    /// operators (e.g. `~`) and stray symbols that aren't operators at
    /// all (e.g. `&`, `|`); the message wording must reflect that.
    UnknownSymbolInRequirement {
        symbol: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` appeared at the top level (level 0 with no enclosing block to
    /// gate). `req` only makes sense as a child of the block it gates.
    RequirementAtTopLevel {
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` compared two expressions of different declared kinds.
    RequirementTypeMismatch {
        left: cuentitos_common::ValueKind,
        right: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` used an ordering operator (`< <= > >=`) on a kind that doesn't
    /// support ordering. Carries the operator so the diagnostic can echo the
    /// exact symbol the author typed.
    NonOrderedComparison {
        operator: cuentitos_common::ComparisonOperator,
        kind: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `req` compared two operands of different declared kinds, naming each
    /// operand's source token and kind (e.g. `cannot compare bool
    /// 'door_open' with int '1'`). Distinct from
    /// [`RequirementTypeMismatch`](Self::RequirementTypeMismatch), which
    /// reports kinds only and covers a type clash inside one operand's
    /// arithmetic.
    RequirementComparisonTypeMismatch {
        left_kind: cuentitos_common::ValueKind,
        left_token: String,
        right_kind: cuentitos_common::ValueKind,
        right_token: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// An arithmetic subexpression operates on a non-numeric kind.
    NonNumericArithmetic {
        kind: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `and`/`or` had a bare arithmetic operand (left or right) instead
    /// of a comparison.
    LogicalBareIntegerOperand {
        operator: crate::boolean_expression::LogicalKeyword,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `not` had a bare arithmetic operand instead of a comparison.
    LogicalBareIntegerOperandOfNot {
        file: Option<PathBuf>,
        line: usize,
    },
    /// `and`/`or` had no left operand. `source` is the trimmed condition.
    LogicalMissingLeftOperand {
        operator: crate::boolean_expression::LogicalKeyword,
        source: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `and`/`or` had no right operand. `source` is the trimmed condition.
    LogicalMissingRightOperand {
        operator: crate::boolean_expression::LogicalKeyword,
        source: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// `not` had no operand. `source` is the trimmed condition.
    LogicalMissingNotOperand {
        source: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// Parens in a `req` condition don't balance.
    UnbalancedParentheses {
        source: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A reserved keyword (e.g. `and`, `or`, `not`) was used as a variable
    /// name in a `--- variables` declaration.
    ReservedKeyword {
        name: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A literal in a `req` arithmetic subexpression exceeded the integer
    /// range. Carries the offending literal text.
    RequirementLiteralOverflow {
        literal: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A literal in a `set` RHS exceeded the integer range. Carries the
    /// offending literal text. Parallel to
    /// [`RequirementLiteralOverflow`](Self::RequirementLiteralOverflow)
    /// so the two sibling parsers produce parallel diagnostics.
    SetLiteralOverflow {
        literal: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A literal in a variables-block default expression exceeded the
    /// integer range. Carries both the variable being defined and the
    /// offending literal text — parallel to
    /// [`SetLiteralOverflow`](Self::SetLiteralOverflow) and
    /// [`RequirementLiteralOverflow`](Self::RequirementLiteralOverflow)
    /// so the three parse-time arithmetic sites produce wording that
    /// only differs by which expression context they name. Binary
    /// overflow at constant-fold time (e.g. `i64::MAX + 1`) keeps using
    /// [`IntegerOverflow`](Self::IntegerOverflow); this variant fires
    /// only when a *single* literal is the cause.
    DefaultLiteralOverflow {
        variable: String,
        literal: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A typed variable's default evaluated to a value of the wrong kind
    /// (e.g. an `int` literal or a reference to an `int` variable used as a
    /// `bool` default). Carries the declared kind, the offending token text,
    /// and the kind that token actually has so the diagnostic can name both —
    /// parallel in shape to [`SetTypeMismatch`](Self::SetTypeMismatch) and
    /// [`RequirementTypeMismatch`](Self::RequirementTypeMismatch), differing
    /// only in that it names a default declaration.
    DefaultTypeMismatch {
        variable: String,
        expected: cuentitos_common::ValueKind,
        found_token: String,
        found: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A float default contained a numeric token that wasn't a valid
    /// `<digits>.<digits>` literal (`.5`, `1.`, `1e3`, a bare integer).
    /// Carries the offending text.
    InvalidFloatLiteral {
        literal: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A float default constant-folded to a non-finite value — its magnitude
    /// exceeded the largest finite `f64`. Distinct from the integer
    /// [`IntegerOverflow`](Self::IntegerOverflow): floats overflow to infinity
    /// rather than wrapping, and the wording names the float context.
    FloatOverflow {
        variable: String,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A float default referenced a non-float value (an int literal or a
    /// reference to an earlier non-float variable). Unlike
    /// [`DefaultTypeMismatch`](Self::DefaultTypeMismatch), the float wording
    /// names a "float expression" and does not quote the operands, matching
    /// the float compatibility spec.
    FloatDefaultTypeMismatch {
        variable: String,
        found_token: String,
        found: cuentitos_common::ValueKind,
        file: Option<PathBuf>,
        line: usize,
    },
    /// A variable default used a logical operator (`and`/`or`/`not`). Boolean
    /// expressions belong in `req`, not in a default; defaults are limited to
    /// literals and references to earlier variables.
    LogicalOperatorInDefault {
        file: Option<PathBuf>,
        line: usize,
    },
    /// `==` appeared in a `req` comparison. Cuentitos spells equality
    /// as a single `=`; the doubled form is the most common typo from
    /// C/Python users and gets a dedicated message so the author isn't
    /// left guessing at a generic "malformed expression" diagnostic.
    DoubleEqualsInRequirement {
        file: Option<PathBuf>,
        line: usize,
    },
    /// A `req` boolean condition nested deeper than the parser's
    /// recursion cap.
    ExpressionTooDeep {
        file: Option<PathBuf>,
        line: usize,
    },
    MultipleErrors {
        errors: Vec<ParseError>,
    },
}

/// Render the prefix used in `Display` error lines: the script's file name, or
/// a neutral placeholder when no path is available.
fn file_prefix(file: &Option<PathBuf>) -> &str {
    file.as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("<script>")
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { file, line } => {
                write!(f, "{}:{}: ERROR: Unexpected token", file_prefix(file), line)
            }
            ParseError::UnexpectedEndOfFile { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Unexpected end of file",
                    file_prefix(file),
                    line
                )
            }
            ParseError::InvalidIndentation {
                message,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Invalid indentation: {}",
                    file_prefix(file),
                    line,
                    message
                )
            }
            ParseError::SectionWithoutTitle { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Section without title: found empty section title.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::InvalidSectionHierarchy {
                message,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Invalid section hierarchy: {}",
                    file_prefix(file),
                    line,
                    message
                )
            }
            ParseError::DuplicateSectionName {
                name,
                parent_name,
                file,
                line,
                previous_line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Duplicate section name: '{}' already exists at this level under '{}'. Previously defined at line {}.",
                    file_prefix(file),
                    line,
                    name,
                    parent_name,
                    previous_line
                )
            }
            ParseError::InvalidGoToSection {
                message,
                file,
                line,
            } => {
                write!(f, "{}:{}: ERROR: {}", file_prefix(file), line, message)
            }
            ParseError::SectionNotFound { path, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Section not found: {}",
                    file_prefix(file),
                    line,
                    path
                )
            }
            ParseError::NavigationAboveRoot { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Cannot navigate above root level",
                    file_prefix(file),
                    line
                )
            }
            ParseError::InvalidSectionName {
                message,
                name,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: {}: {}",
                    file_prefix(file),
                    line,
                    message,
                    name
                )
            }
            ParseError::EmptySection { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Section must contain at least one block: {}",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::OptionsWithoutParent { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Options must have a parent",
                    file_prefix(file),
                    line
                )
            }
            ParseError::UnterminatedVariablesBlock { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Unterminated '--- variables' block: missing closing '---'.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::DuplicateVariable {
                name,
                previous_line,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Duplicate variable name: '{}' already declared. Previously declared at line {}.",
                    file_prefix(file),
                    line,
                    name,
                    previous_line
                )
            }
            ParseError::MalformedDefaultExpression { expr, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed default expression: '{}'.",
                    file_prefix(file),
                    line,
                    expr
                )
            }
            ParseError::DivisionByZero {
                variable,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Division by zero in default expression for '{}'.",
                    file_prefix(file),
                    line,
                    variable
                )
            }
            ParseError::IntegerOverflow {
                variable,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Integer overflow in default expression for '{}'.",
                    file_prefix(file),
                    line,
                    variable
                )
            }
            ParseError::InvalidFloatLiteral {
                literal,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Invalid float literal: '{}'. Float literals must be written as <digits>.<digits> (e.g. '1.5').",
                    file_prefix(file),
                    line,
                    literal
                )
            }
            ParseError::FloatOverflow {
                variable,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Float overflow in default expression for '{}'.",
                    file_prefix(file),
                    line,
                    variable
                )
            }
            ParseError::FloatDefaultTypeMismatch {
                variable,
                found_token,
                found,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch: default for float {} must be a float expression, but {} is {}.",
                    file_prefix(file),
                    line,
                    variable,
                    found_token,
                    found.keyword()
                )
            }
            ParseError::InvalidVariableName { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Invalid variable name: '{}'. Variable names must start with a letter or underscore.",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::UndefinedVariableReference { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Undefined variable: '{}'.",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::ForwardVariableReference { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Forward reference: variable '{}' referenced before declaration.",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::SelfReferenceInDefault { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Self-reference in default: variable '{}' references itself.",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::IndentedVariableDeclaration {
                content,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed variable declaration: '{}'. Declarations must not be indented.",
                    file_prefix(file),
                    line,
                    content
                )
            }
            ParseError::MissingVariableName { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed variable declaration: missing variable name.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::MalformedVariableDeclaration {
                content,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed variable declaration: '{}'. Expected 'int <name> [= <expr>]'.",
                    file_prefix(file),
                    line,
                    content
                )
            }
            ParseError::MalformedSetExpression {
                expression,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed 'set' statement: '{}'. ('set' is reserved at the start of a line; indent or rephrase to use it in narrative text.)",
                    file_prefix(file),
                    line,
                    expression
                )
            }
            ParseError::SetTypeMismatch {
                variable,
                expected,
                found,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch in 'set': variable '{}' has type {} but expression has type {}.",
                    file_prefix(file),
                    line,
                    variable,
                    expected,
                    found
                )
            }
            ParseError::FloatSetTypeMismatch {
                variable,
                found_token,
                found,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch: 'set' expression for float {} must be a float expression, but {} is {}.",
                    file_prefix(file),
                    line,
                    variable,
                    found_token,
                    found.keyword()
                )
            }
            ParseError::NonNumericAssignment {
                variable,
                kind,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Compound assignment requires a numeric variable; '{}' has type {}.",
                    file_prefix(file),
                    line,
                    variable,
                    kind
                )
            }
            ParseError::MalformedRequirementExpression {
                expression,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Malformed expression in 'req': '{}'.",
                    file_prefix(file),
                    line,
                    expression
                )
            }
            ParseError::UnknownSymbolInRequirement { symbol, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Unknown operator '{}' in 'req'.",
                    file_prefix(file),
                    line,
                    symbol
                )
            }
            ParseError::RequirementAtTopLevel { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Top-level 'req' has no parent block.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::RequirementTypeMismatch {
                left,
                right,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch in 'req': left side has type {} but right side has type {}.",
                    file_prefix(file),
                    line,
                    left,
                    right
                )
            }
            ParseError::NonOrderedComparison {
                operator,
                kind,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Ordering operator '{}' is not supported on {} values.",
                    file_prefix(file),
                    line,
                    operator.symbol(),
                    kind.keyword()
                )
            }
            ParseError::RequirementComparisonTypeMismatch {
                left_kind,
                left_token,
                right_kind,
                right_token,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch: cannot compare {} '{}' with {} '{}'.",
                    file_prefix(file),
                    line,
                    left_kind.keyword(),
                    left_token,
                    right_kind.keyword(),
                    right_token
                )
            }
            ParseError::NonNumericArithmetic { kind, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Arithmetic operator is not supported for type {}.",
                    file_prefix(file),
                    line,
                    kind
                )
            }
            ParseError::LogicalBareIntegerOperand {
                operator,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Logical operator '{}' expects a comparison as each operand, not an integer expression.",
                    file_prefix(file),
                    line,
                    operator.as_str()
                )
            }
            ParseError::LogicalBareIntegerOperandOfNot { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Logical operator 'not' expects a comparison as its operand, not an integer expression.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::LogicalMissingLeftOperand {
                operator,
                source,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Missing left operand for '{}' in 'req': '{}'.",
                    file_prefix(file),
                    line,
                    operator.as_str(),
                    source
                )
            }
            ParseError::LogicalMissingRightOperand {
                operator,
                source,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Missing right operand for '{}' in 'req': '{}'.",
                    file_prefix(file),
                    line,
                    operator.as_str(),
                    source
                )
            }
            ParseError::LogicalMissingNotOperand { source, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Missing operand for 'not' in 'req': '{}'.",
                    file_prefix(file),
                    line,
                    source
                )
            }
            ParseError::UnbalancedParentheses { source, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Unbalanced parentheses in 'req': '{}'.",
                    file_prefix(file),
                    line,
                    source
                )
            }
            ParseError::ReservedKeyword { name, file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Reserved keyword '{}' cannot be used as a variable name.",
                    file_prefix(file),
                    line,
                    name
                )
            }
            ParseError::RequirementLiteralOverflow {
                literal,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Integer overflow in 'req' expression: literal '{}' exceeds the integer range.",
                    file_prefix(file),
                    line,
                    literal
                )
            }
            ParseError::SetLiteralOverflow {
                literal,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Integer overflow in 'set' expression: literal '{}' exceeds the integer range.",
                    file_prefix(file),
                    line,
                    literal
                )
            }
            ParseError::DefaultLiteralOverflow {
                variable,
                literal,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Integer overflow in default expression for '{}': literal '{}' exceeds the integer range.",
                    file_prefix(file),
                    line,
                    variable,
                    literal
                )
            }
            ParseError::DefaultTypeMismatch {
                variable,
                expected,
                found_token,
                found,
                file,
                line,
            } => {
                write!(
                    f,
                    "{}:{}: ERROR: Type mismatch: default for {} '{}' must be a {}, but '{}' is {}.",
                    file_prefix(file),
                    line,
                    expected.keyword(),
                    variable,
                    expected.keyword(),
                    found_token,
                    found.keyword()
                )
            }
            ParseError::LogicalOperatorInDefault { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Logical operators (and/or/not) are not allowed in variable defaults; use 'req' for boolean expressions.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::DoubleEqualsInRequirement { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: Use '=' for equality, not '=='.",
                    file_prefix(file),
                    line
                )
            }
            ParseError::ExpressionTooDeep { file, line } => {
                write!(
                    f,
                    "{}:{}: ERROR: 'req' expression nests too deeply (max {} levels).",
                    file_prefix(file),
                    line,
                    crate::boolean_expression::MAX_EXPRESSION_DEPTH
                )
            }
            ParseError::MultipleErrors { errors } => {
                for (i, error) in errors.iter().enumerate() {
                    write!(f, "{}", error)?;
                    if i < errors.len() - 1 {
                        writeln!(f)?;
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            option_parser: OptionParser::new(),
            go_to_section_parser: GoToSectionParser::new(),
            ..Self::default()
        }
    }

    pub fn with_file(file_path: PathBuf) -> Self {
        Self {
            file_path: Some(file_path),
            line_parser: LineParser::new(),
            section_parser: SectionParser::new(),
            option_parser: OptionParser::new(),
            go_to_section_and_back_parser: GoToSectionAndBackParser::new(),
            go_to_section_parser: GoToSectionParser::new(),
            ..Self::default()
        }
    }

    /// Helper to collect an error and skip the current line
    fn collect_error_and_skip(&mut self, error: ParseError, context: &mut ParserContext) {
        self.errors.push(error);
        context.current_line += 1;
    }

    fn mark_non_option_child(&mut self, parent_id: Option<BlockId>) {
        if let Some(parent_id) = parent_id {
            self.seen_non_option_by_parent.insert(parent_id, true);
        }
    }

    pub fn parse<A>(&mut self, script: A) -> Result<(Database, Vec<Warning>), ParseError>
    where
        A: AsRef<str>,
    {
        // Clear state from previous parse
        self.warnings.clear();
        self.errors.clear();
        self.last_block_at_level.clear();
        self.last_section_at_level.clear();
        self.seen_non_option_by_parent.clear();
        self.section_names_by_parent.clear();
        self.section_ids_by_parent.clear();
        self.goto_paths.clear();

        let mut context = if let Some(file_path) = &self.file_path {
            ParserContext::with_file(file_path.clone())
        } else {
            ParserContext::new()
        };

        // Add Start block
        let start_block = Block::new(BlockType::Start, None, 0);
        let start_id = context.database.add_block(start_block);
        self.last_block_at_level.push(start_id);
        self.last_section_at_level.push(start_id); // Start can be parent of top-level sections

        // Collect lines so we can consume a leading `--- variables` block without
        // disturbing line-number accounting for the rest of the script.
        let collected: Vec<&str> = script.as_ref().lines().collect();
        let mut skip_until_index: usize = 0;

        // Find the first non-empty, non-comment line. If it is `--- variables`,
        // parse that block before the main pass. Only a leading variables block
        // is supported; anything else is treated as regular content and a
        // mid-file occurrence surfaces as a warning in the main pass.
        for (i, line) in collected.iter().copied().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || Self::is_comment(line) {
                continue;
            }
            if trimmed == "--- variables" {
                let outcome = crate::parsers::variables_parser::parse_variables_block(
                    &collected,
                    i,
                    &mut context.database,
                    &self.file_path,
                );
                // `consumed_lines` always covers the full block span (or the
                // rest of the file when the block is unterminated), so the
                // main pass can resume cleanly in both success and error
                // cases.
                skip_until_index = i + outcome.consumed_lines;
                for error in outcome.errors {
                    self.errors.push(error);
                }
            }
            break;
        }

        // Iterate through each line, accounting for any block consumed above.
        for (line_index, line) in collected.iter().copied().enumerate() {
            if line_index < skip_until_index {
                context.current_line += 1;
                continue;
            }
            // `--- variables` is only valid as the first directive in the
            // file. Anything later is almost certainly a user mistake.
            if line.trim() == "--- variables" {
                self.warnings.push(Warning {
                    message:
                        "'--- variables' block must appear at the start of the file; treating as content."
                            .to_string(),
                    file: self.file_path.clone(),
                    line: context.current_line,
                });
            }
            // Skip comment lines
            if Self::is_comment(line) {
                context.current_line += 1;
                continue;
            }

            let (level, content) = match self.parse_indentation(line, &context) {
                Ok(result) => result,
                Err(e) => {
                    self.collect_error_and_skip(e, &mut context);
                    continue;
                }
            };

            if content.trim().is_empty() {
                context.current_line += 1;
                continue; // Skip empty lines
            }

            // Try to parse as section first
            let section_result = match self.section_parser.parse(content.trim(), &mut context) {
                Ok(result) => result,
                Err(e) => {
                    self.collect_error_and_skip(e, &mut context);
                    continue;
                }
            };

            if let Some(mut section_result) = section_result {
                // Check for leading/trailing whitespace in section display name
                let original_display = section_result.display_name.clone();
                let trimmed_display = original_display.trim();
                if trimmed_display != original_display {
                    self.warnings.push(Warning {
                        message: format!(
                            "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                            original_display, trimmed_display
                        ),
                        file: self.file_path.clone(),
                        line: context.current_line,
                    });
                    // Update display name; only update id if it matched the original display
                    section_result.display_name = trimmed_display.to_string();
                    if section_result.id_is_implicit {
                        section_result.id = trimmed_display.to_string();
                    }
                }

                // This is a section
                let parent_id = if level == 0 {
                    Some(start_id)
                } else if level <= self.last_section_at_level.len() {
                    // Pop section levels until we reach the parent level
                    while self.last_section_at_level.len() > level {
                        self.last_section_at_level.pop();
                    }
                    self.last_section_at_level.last().copied()
                } else {
                    self.collect_error_and_skip(
                        ParseError::InvalidIndentation {
                            message: format!("found {} spaces in: {}", level * 2, content),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        },
                        &mut context,
                    );
                    continue;
                };

                // Validate orphaned subsections: if hash_count > 1 (subsection) and parent is start_id, that's an error
                if section_result.hash_count > 1 && parent_id == Some(start_id) {
                    self.collect_error_and_skip(
                        ParseError::InvalidSectionHierarchy {
                            message: "found sub-section without parent section.".to_string(),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        },
                        &mut context,
                    );
                    continue;
                }

                // Check for duplicate section display names
                let names_map = self.section_names_by_parent.entry(parent_id).or_default();

                if let Some(&previous_line) = names_map.get(&section_result.display_name) {
                    // Get parent's display name for error message
                    let parent_name = if let Some(pid) = parent_id {
                        if pid == start_id {
                            "<root>".to_string()
                        } else {
                            match &context.database.blocks[pid].block_type {
                                BlockType::Section(section_id) => {
                                    let section = &context.database.sections[*section_id];
                                    context.database.strings[section.name].clone()
                                }
                                _ => "<root>".to_string(),
                            }
                        }
                    } else {
                        "<root>".to_string()
                    };

                    // Collect error and skip adding this block
                    self.collect_error_and_skip(
                        ParseError::DuplicateSectionName {
                            name: section_result.display_name.clone(),
                            parent_name,
                            file: self.file_path.clone(),
                            line: context.current_line,
                            previous_line,
                        },
                        &mut context,
                    );
                    continue;
                }

                // Record this section display name
                names_map.insert(section_result.display_name.clone(), context.current_line);

                // Check for duplicate section ids
                let ids_map = self.section_ids_by_parent.entry(parent_id).or_default();
                if let Some(&previous_line) = ids_map.get(&section_result.id) {
                    let parent_name = if let Some(pid) = parent_id {
                        if pid == start_id {
                            "<root>".to_string()
                        } else {
                            match &context.database.blocks[pid].block_type {
                                BlockType::Section(section_id) => {
                                    let section = &context.database.sections[*section_id];
                                    context.database.strings[section.name].clone()
                                }
                                _ => "<root>".to_string(),
                            }
                        }
                    } else {
                        "<root>".to_string()
                    };

                    self.collect_error_and_skip(
                        ParseError::DuplicateSectionName {
                            name: section_result.id.clone(),
                            parent_name,
                            file: self.file_path.clone(),
                            line: context.current_line,
                            previous_line,
                        },
                        &mut context,
                    );
                    continue;
                }

                // Record this section id
                ids_map.insert(section_result.id.clone(), context.current_line);

                // Build the full display path for this section
                let path_string = self.build_section_path_during_parse(
                    &context.database,
                    &section_result.display_name,
                    level,
                );
                // Build the full id path for this section
                let id_path_string = self.build_section_id_path_during_parse(
                    &context.database,
                    &section_result.id,
                    level,
                );

                // Add name, id, and paths to strings database
                let name_string_id = context
                    .database
                    .add_string(section_result.display_name.clone());
                let id_string_id = if section_result.id_is_implicit {
                    name_string_id
                } else {
                    context.database.add_string(section_result.id.clone())
                };
                let path_string_id = context.database.add_string(path_string);
                let id_path_string_id = context.database.add_string(id_path_string);

                // Create a placeholder block first to get the block_id
                let block = Block::with_line(
                    BlockType::Section(0), // Temporary, will be updated
                    parent_id,
                    level,
                    context.current_line,
                );
                let block_id = context.database.add_block(block);

                // Now create the Section and add it to the database
                let section = cuentitos_common::Section::new(
                    block_id,
                    name_string_id,
                    id_string_id,
                    path_string_id,
                    id_path_string_id,
                );
                let section_id = context.database.add_section(section);

                // Update the block with the correct section_id
                context.database.blocks[block_id].block_type = BlockType::Section(section_id);

                // Update both stacks
                if level >= self.last_block_at_level.len() {
                    self.last_block_at_level.push(block_id);
                } else {
                    self.last_block_at_level[level] = block_id;
                }

                if level >= self.last_section_at_level.len() {
                    self.last_section_at_level.push(block_id);
                } else {
                    self.last_section_at_level[level] = block_id;
                }

                // Mark that parent has seen a non-option child
                self.mark_non_option_child(parent_id);
            } else {
                // Try to parse as go-to-section-and-back first (before go-to-section)
                let go_to_and_back_result = match self
                    .go_to_section_and_back_parser
                    .parse(content.trim(), &mut context)
                {
                    Ok(result) => result,
                    Err(e) => {
                        self.collect_error_and_skip(e, &mut context);
                        continue;
                    }
                };

                if let Some(go_to_and_back_result) = go_to_and_back_result {
                    // Check for leading/trailing whitespace in goto path
                    let trimmed_path = go_to_and_back_result.path.trim();
                    let final_path = if trimmed_path != go_to_and_back_result.path {
                        self.warnings.push(Warning {
                            message: format!(
                                "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                                go_to_and_back_result.path, trimmed_path
                            ),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        });
                        trimmed_path.to_string()
                    } else {
                        go_to_and_back_result.path
                    };

                    // This is a go-to-section-and-back command. Resolve
                    // the parent block via the shared helper.
                    let parent_id =
                        match self.resolve_parent_id(level, content, context.current_line) {
                            Ok(parent_id) => parent_id,
                            Err(err) => {
                                self.collect_error_and_skip(err, &mut context);
                                continue;
                            }
                        };

                    // Create GoToAndBack block with placeholder SectionId
                    // This will be resolved in the validation pass
                    let block = Block::with_line(
                        BlockType::GoToAndBack(0), // Placeholder SectionId, will be resolved
                        parent_id,
                        level,
                        context.current_line,
                    );
                    let block_id = context.database.add_block(block);

                    // Store the path for later resolution
                    self.goto_paths.insert(block_id, final_path);

                    // Update last block at this level
                    if level >= self.last_block_at_level.len() {
                        self.last_block_at_level.push(block_id);
                    } else {
                        self.last_block_at_level[level] = block_id;
                    }

                    // Mark that parent has seen a non-option child
                    self.mark_non_option_child(parent_id);
                } else {
                    // Try to parse as go-to-section
                    let go_to_result = match self
                        .go_to_section_parser
                        .parse(content.trim(), &mut context)
                    {
                        Ok(result) => result,
                        Err(e) => {
                            self.collect_error_and_skip(e, &mut context);
                            continue;
                        }
                    };

                    if let Some(go_to_result) = go_to_result {
                        // Check for leading/trailing whitespace in goto path
                        let trimmed_path = go_to_result.path.trim();
                        let final_path = if trimmed_path != go_to_result.path {
                            self.warnings.push(Warning {
                            message: format!(
                                "Section name has leading/trailing whitespace: '{}'. Trimmed to '{}'",
                                go_to_result.path, trimmed_path
                            ),
                            file: self.file_path.clone(),
                            line: context.current_line,
                        });
                            trimmed_path.to_string()
                        } else {
                            go_to_result.path
                        };

                        // This is a go-to-section command
                        // Find parent block: check if there's a section at this level first
                        let parent_id = if level < self.last_section_at_level.len() {
                            // There's a section at this level, use it as parent
                            Some(self.last_section_at_level[level])
                        } else if level == 0 {
                            // At level 0 with no section, use first block (Start or a Section)
                            self.last_block_at_level.first().copied()
                        } else if level <= self.last_block_at_level.len() {
                            // No section at this level, use last block at previous level
                            self.last_block_at_level.get(level - 1).copied()
                        } else {
                            self.collect_error_and_skip(
                                ParseError::InvalidIndentation {
                                    message: format!("found {} spaces in: {}", level * 2, content),
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        };

                        // Create GoTo block with placeholder SectionId
                        // This will be resolved in the validation pass
                        let block = Block::with_line(
                            BlockType::GoTo(0), // Placeholder SectionId, will be resolved
                            parent_id,
                            level,
                            context.current_line,
                        );
                        let block_id = context.database.add_block(block);

                        // Store the path for later resolution
                        self.goto_paths.insert(block_id, final_path);

                        // Update last block at this level
                        if level >= self.last_block_at_level.len() {
                            self.last_block_at_level.push(block_id);
                        } else {
                            self.last_block_at_level[level] = block_id;
                        }

                        // Mark that parent has seen a non-option child
                        self.mark_non_option_child(parent_id);
                    } else if Self::looks_like_set_line(content.trim()) {
                        // Parse `set <var> <op> <expr>`. Resolve LHS / RHS
                        // identifiers against declared variables; surface
                        // malformed expressions and undeclared references
                        // as parse-time errors.
                        match crate::parsers::set_parser::parse_set(
                            content.trim(),
                            &context.database,
                        ) {
                            Ok(parsed) => {
                                let parent_id = match self.resolve_parent_id(
                                    level,
                                    content,
                                    context.current_line,
                                ) {
                                    Ok(parent_id) => parent_id,
                                    Err(err) => {
                                        self.collect_error_and_skip(err, &mut context);
                                        continue;
                                    }
                                };

                                let set_id =
                                    context
                                        .database
                                        .add_set(cuentitos_common::SetStatement::new(
                                            parsed.variable_id,
                                            parsed.operator,
                                            parsed.expression,
                                        ));
                                let block = Block::with_line(
                                    BlockType::Set(set_id),
                                    parent_id,
                                    level,
                                    context.current_line,
                                );
                                let block_id = context.database.add_block(block);

                                if level >= self.last_block_at_level.len() {
                                    self.last_block_at_level.push(block_id);
                                } else {
                                    self.last_block_at_level[level] = block_id;
                                }

                                self.mark_non_option_child(parent_id);
                            }
                            Err(set_err) => {
                                use crate::parsers::set_parser::SetParseError;
                                let parse_error = match set_err {
                                    SetParseError::UndefinedVariable { name } => {
                                        ParseError::UndefinedVariableReference {
                                            name,
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                    SetParseError::MalformedExpression { expression } => {
                                        ParseError::MalformedSetExpression {
                                            expression,
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                    SetParseError::InvalidLhs { name } => {
                                        ParseError::InvalidVariableName {
                                            name,
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                    SetParseError::MissingLhs
                                    | SetParseError::MissingAssignment
                                    | SetParseError::MissingRhs => {
                                        ParseError::MalformedSetExpression {
                                            expression: content.trim().to_string(),
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                    SetParseError::TypeMismatch {
                                        variable,
                                        expected,
                                        found,
                                    } => ParseError::SetTypeMismatch {
                                        variable,
                                        expected,
                                        found,
                                        file: self.file_path.clone(),
                                        line: context.current_line,
                                    },
                                    SetParseError::FloatTypeMismatch {
                                        variable,
                                        found_token,
                                        found,
                                    } => ParseError::FloatSetTypeMismatch {
                                        variable,
                                        found_token,
                                        found,
                                        file: self.file_path.clone(),
                                        line: context.current_line,
                                    },
                                    SetParseError::NonNumericAssignment { variable, kind } => {
                                        ParseError::NonNumericAssignment {
                                            variable,
                                            kind,
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                    SetParseError::LiteralOverflow { literal } => {
                                        ParseError::SetLiteralOverflow {
                                            literal,
                                            file: self.file_path.clone(),
                                            line: context.current_line,
                                        }
                                    }
                                };
                                self.collect_error_and_skip(parse_error, &mut context);
                                continue;
                            }
                        }
                    } else if Self::looks_like_requirement_line(content.trim()) {
                        // `req` is a parent-gating statement: parse the
                        // condition, resolve identifiers, and surface
                        // malformed/undeclared/unknown-operator errors at
                        // parse time. A `req` at level 0 has no narrative
                        // block to gate (its only ancestor is START), so
                        // it's a parse-time error.
                        if level == 0 {
                            self.collect_error_and_skip(
                                ParseError::RequirementAtTopLevel {
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        }

                        match crate::parsers::requirement_parser::parse_requirement(
                            content.trim(),
                            &context.database,
                        ) {
                            Ok(parsed) => {
                                let parent_id = match self.resolve_parent_id(
                                    level,
                                    content,
                                    context.current_line,
                                ) {
                                    Ok(parent_id) => parent_id,
                                    Err(err) => {
                                        self.collect_error_and_skip(err, &mut context);
                                        continue;
                                    }
                                };

                                let requirement_id =
                                    context.database.add_requirement(parsed.expression);
                                let block = Block::with_line(
                                    BlockType::Requirement(requirement_id),
                                    parent_id,
                                    level,
                                    context.current_line,
                                );
                                let block_id = context.database.add_block(block);

                                if level >= self.last_block_at_level.len() {
                                    self.last_block_at_level.push(block_id);
                                } else {
                                    self.last_block_at_level[level] = block_id;
                                }

                                self.mark_non_option_child(parent_id);
                            }
                            Err(requirement_err) => {
                                use crate::parsers::requirement_parser::RequirementParseError;
                                let file = self.file_path.clone();
                                let line = context.current_line;
                                let parse_error = match requirement_err {
                                    RequirementParseError::UndefinedVariable { name } => {
                                        ParseError::UndefinedVariableReference { name, file, line }
                                    }
                                    RequirementParseError::MalformedExpression { expression } => {
                                        ParseError::MalformedRequirementExpression {
                                            expression,
                                            file,
                                            line,
                                        }
                                    }
                                    RequirementParseError::MissingCondition => {
                                        ParseError::MalformedRequirementExpression {
                                            expression: content.trim().to_string(),
                                            file,
                                            line,
                                        }
                                    }
                                    RequirementParseError::UnknownSymbol { symbol } => {
                                        ParseError::UnknownSymbolInRequirement {
                                            symbol,
                                            file,
                                            line,
                                        }
                                    }
                                    RequirementParseError::TypeMismatch { left, right } => {
                                        ParseError::RequirementTypeMismatch {
                                            left,
                                            right,
                                            file,
                                            line,
                                        }
                                    }
                                    RequirementParseError::ComparisonTypeMismatch {
                                        left_kind,
                                        left_token,
                                        right_kind,
                                        right_token,
                                    } => ParseError::RequirementComparisonTypeMismatch {
                                        left_kind,
                                        left_token,
                                        right_kind,
                                        right_token,
                                        file,
                                        line,
                                    },
                                    RequirementParseError::NonOrderedComparison {
                                        operator,
                                        kind,
                                    } => ParseError::NonOrderedComparison {
                                        operator,
                                        kind,
                                        file,
                                        line,
                                    },
                                    RequirementParseError::NonNumericArithmetic { kind } => {
                                        ParseError::NonNumericArithmetic { kind, file, line }
                                    }
                                    RequirementParseError::LogicalBareIntegerOperand {
                                        operator,
                                    } => ParseError::LogicalBareIntegerOperand {
                                        operator,
                                        file,
                                        line,
                                    },
                                    RequirementParseError::LogicalBareIntegerOperandOfNot => {
                                        ParseError::LogicalBareIntegerOperandOfNot { file, line }
                                    }
                                    RequirementParseError::LogicalMissingLeftOperand {
                                        operator,
                                        source,
                                    } => ParseError::LogicalMissingLeftOperand {
                                        operator,
                                        source,
                                        file,
                                        line,
                                    },
                                    RequirementParseError::LogicalMissingRightOperand {
                                        operator,
                                        source,
                                    } => ParseError::LogicalMissingRightOperand {
                                        operator,
                                        source,
                                        file,
                                        line,
                                    },
                                    RequirementParseError::LogicalMissingNotOperand { source } => {
                                        ParseError::LogicalMissingNotOperand { source, file, line }
                                    }
                                    RequirementParseError::LogicalUnbalancedParentheses {
                                        source,
                                    } => ParseError::UnbalancedParentheses { source, file, line },
                                    RequirementParseError::LiteralOverflow { literal } => {
                                        ParseError::RequirementLiteralOverflow {
                                            literal,
                                            file,
                                            line,
                                        }
                                    }
                                    RequirementParseError::ExpressionTooDeep => {
                                        ParseError::ExpressionTooDeep { file, line }
                                    }
                                    RequirementParseError::DoubleEquals => {
                                        ParseError::DoubleEqualsInRequirement { file, line }
                                    }
                                };
                                self.collect_error_and_skip(parse_error, &mut context);
                                continue;
                            }
                        }
                    } else if OptionParser::is_option_line(content.trim()) {
                        // Parse as option
                        let result = match self.option_parser.parse(content.trim(), &mut context) {
                            Ok(result) => result,
                            Err(e) => {
                                self.collect_error_and_skip(e, &mut context);
                                continue;
                            }
                        };

                        // Find parent block
                        let parent_id = if level < self.last_section_at_level.len() {
                            // There's a section at this level, use it as parent
                            Some(self.last_section_at_level[level])
                        } else if level == 0 {
                            // At level 0 with no section, use first block (Start or a Section)
                            self.last_block_at_level.first().copied()
                        } else if level <= self.last_block_at_level.len() {
                            // No section at this level, use last block at previous level
                            self.last_block_at_level.get(level - 1).copied()
                        } else {
                            self.collect_error_and_skip(
                                ParseError::InvalidIndentation {
                                    message: format!("found {} spaces in: {}", level * 2, content),
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        };

                        // Options at root level (level 0) or without a parent are invalid
                        if level == 0 || parent_id.is_none() {
                            self.collect_error_and_skip(
                                ParseError::OptionsWithoutParent {
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        }

                        let parent_id = parent_id.unwrap();
                        let parent_block = &context.database.blocks[parent_id];

                        // Parent must be a String or Option block
                        if !matches!(
                            parent_block.block_type,
                            BlockType::String(_) | BlockType::Option(_)
                        ) {
                            self.collect_error_and_skip(
                                ParseError::OptionsWithoutParent {
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        }

                        // Options cannot have non-option siblings before them
                        if self
                            .seen_non_option_by_parent
                            .get(&parent_id)
                            .copied()
                            .unwrap_or(false)
                        {
                            self.collect_error_and_skip(
                                ParseError::OptionsWithoutParent {
                                    file: self.file_path.clone(),
                                    line: context.current_line,
                                },
                                &mut context,
                            );
                            continue;
                        }

                        // Create option block
                        let string_id = context.database.add_string(result.text);
                        let block = Block::with_line(
                            BlockType::Option(string_id),
                            Some(parent_id),
                            level,
                            context.current_line,
                        );
                        let block_id = context.database.add_block(block);

                        // Update last block at this level
                        if self.last_block_at_level.len() > level {
                            self.last_block_at_level.truncate(level);
                        }
                        if level >= self.last_block_at_level.len() {
                            self.last_block_at_level.push(block_id);
                        } else {
                            self.last_block_at_level[level] = block_id;
                        }
                    } else {
                        // Parse as regular string
                        let result = self.line_parser.parse(content.trim(), &mut context)?;

                        // Resolve parent via shared helper. The String branch
                        // returns the indentation error directly (matching
                        // historical behavior) rather than collecting it.
                        let parent_id =
                            self.resolve_parent_id(level, content, context.current_line)?;

                        // Create new block
                        let string_id = context.database.add_string(result.string);
                        let block = Block::with_line(
                            BlockType::String(string_id),
                            parent_id,
                            level,
                            context.current_line,
                        );
                        let block_id = context.database.add_block(block);

                        // Update last block at this level
                        if level >= self.last_block_at_level.len() {
                            self.last_block_at_level.push(block_id);
                        } else {
                            self.last_block_at_level[level] = block_id;
                        }

                        // Mark that parent has seen a non-option child
                        self.mark_non_option_child(parent_id);
                    }
                }
            }

            // Increment line counter after processing each non-empty line
            context.current_line += 1;
        }

        // Add End block (with no parent)
        let end_block = Block::new(BlockType::End, None, 0);
        context.database.add_block(end_block);

        // Run compile-time validation pass
        self.validate_and_resolve(&mut context)?;

        // Check if we collected any errors
        if !self.errors.is_empty() {
            if self.errors.len() == 1 {
                return Err(self.errors.remove(0));
            } else {
                return Err(ParseError::MultipleErrors {
                    errors: self.errors.clone(),
                });
            }
        }

        Ok((context.database, self.warnings.clone()))
    }

    /// Returns true if the line is a comment (starts with // after optional whitespace).
    ///
    /// Comments can appear at any indentation level and are completely ignored by the parser.
    /// Only line-level comments are supported; inline comments (// after content) are not detected.
    fn is_comment(line: &str) -> bool {
        line.trim_start().starts_with("//")
    }

    /// Cheap pre-filter: does this trimmed line begin with the `set` keyword?
    /// Lines that pass through here are then handed to `set_parser::parse_set`
    /// for full validation.
    fn looks_like_set_line(content: &str) -> bool {
        crate::parsers::set_parser::is_set_line(content)
    }

    /// Cheap pre-filter: does this trimmed line begin with the `req` keyword?
    /// Lines that pass through here are then handed to
    /// `requirement_parser::parse_requirement` for full validation.
    fn looks_like_requirement_line(content: &str) -> bool {
        crate::parsers::requirement_parser::is_requirement_line(content)
    }

    /// Find the parent block for a non-section block at `level`. Mutates
    /// `last_block_at_level` (pops back to the new level) so subsequent
    /// blocks at the same level see the right ancestor. Returns
    /// `Err(InvalidIndentation)` if the level skipped a step.
    ///
    /// Shared between the String, GoToAndBack, and Set branches; the
    /// GoTo/Option branches keep a non-mutating `get(level - 1)` lookup
    /// because they don't reset the level stack on visit.
    fn resolve_parent_id(
        &mut self,
        level: usize,
        content: &str,
        line: usize,
    ) -> Result<Option<BlockId>, ParseError> {
        if level < self.last_section_at_level.len() {
            Ok(Some(self.last_section_at_level[level]))
        } else if level == 0 {
            Ok(self.last_block_at_level.first().copied())
        } else if level <= self.last_block_at_level.len() {
            while self.last_block_at_level.len() > level {
                self.last_block_at_level.pop();
            }
            Ok(self.last_block_at_level.last().copied())
        } else {
            Err(ParseError::InvalidIndentation {
                message: format!("found {} spaces in: {}", level * 2, content),
                file: self.file_path.clone(),
                line,
            })
        }
    }

    fn parse_indentation<'a>(
        &self,
        line: &'a str,
        context: &ParserContext,
    ) -> Result<(usize, &'a str), ParseError> {
        let mut spaces = 0;
        for ch in line.chars() {
            match ch {
                ' ' => spaces += 1,
                '\t' => {
                    return Err(ParseError::InvalidIndentation {
                        message: "found tab indentation.".to_string(),
                        file: self.file_path.clone(),
                        line: context.current_line,
                    })
                }
                _ => break,
            }
        }

        // Check if indentation is valid (multiple of 2)
        if spaces % 2 != 0 {
            return Err(ParseError::InvalidIndentation {
                message: format!("found {} spaces.", spaces),
                file: self.file_path.clone(),
                line: context.current_line,
            });
        }

        Ok((spaces / 2, &line[spaces..]))
    }

    /// Validate and resolve all GoToSection blocks
    /// This runs after parsing and performs compile-time checks
    fn validate_and_resolve(&mut self, context: &mut ParserContext) -> Result<(), ParseError> {
        // Build section registry: map section paths to SectionId
        let section_registry = self.build_section_registry(&context.database);

        // Store the registry in the database for runtime use
        context.database.section_registry = section_registry.clone();

        // Validate section names don't contain backslash
        self.validate_section_names(&context.database)?;

        // Only detect empty sections if there were no parse errors
        // (parse errors may have prevented blocks from being created)
        if self.errors.is_empty() {
            self.detect_empty_sections(&context.database)?;
        }

        // Collect GoTo and GoToAndBack blocks first to avoid borrow checker issues
        let goto_blocks: Vec<(BlockId, String, usize, bool)> = context
            .database
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(block_id, block)| match &block.block_type {
                BlockType::GoTo(_) => {
                    let path = self.goto_paths.get(&block_id)?.clone();
                    Some((block_id, path, block.line, false))
                }
                BlockType::GoToAndBack(_) => {
                    let path = self.goto_paths.get(&block_id)?.clone();
                    Some((block_id, path, block.line, true))
                }
                _ => None,
            })
            .collect();

        // Resolve and validate all GoToSection and GoToSectionAndBack blocks
        for (block_id, path, line, is_call_and_back) in goto_blocks {
            // Find the containing section for this block
            let containing_section = self.find_containing_section(&context.database, block_id);

            // Resolve the path
            match self.resolve_path(
                &path,
                containing_section,
                &section_registry,
                &context.database,
                line,
            ) {
                Ok(resolved_type) => {
                    // Update the block with the resolved type
                    // For GoToAndBack, convert GoTo variants to GoToAndBack, keep special variants as-is
                    context.database.blocks[block_id].block_type = if is_call_and_back {
                        match resolved_type {
                            BlockType::GoTo(section_id) => {
                                // Check if the target section has only goto blocks
                                let target_section = &context.database.sections[section_id];
                                let target_block_id = target_section.block_id;

                                if Self::section_has_only_gotos(&context.database, target_block_id)
                                {
                                    let section_name =
                                        &context.database.strings[target_section.name];
                                    let section_line =
                                        context.database.blocks[target_block_id].line;
                                    self.errors.push(ParseError::EmptySection {
                                        name: section_name.clone(),
                                        file: self.file_path.clone(),
                                        line: section_line,
                                    });
                                }

                                BlockType::GoToAndBack(section_id)
                            }
                            BlockType::GoToStart => {
                                self.warnings.push(Warning {
                                    message: "<-> START will not return (restarts from beginning)"
                                        .to_string(),
                                    file: self.file_path.clone(),
                                    line,
                                });
                                BlockType::GoToStart
                            }
                            BlockType::GoToRestart => {
                                self.warnings.push(Warning {
                                    message: "<-> RESTART will not return (clears state and restarts from beginning)".to_string(),
                                    file: self.file_path.clone(),
                                    line,
                                });
                                BlockType::GoToRestart
                            }
                            BlockType::GoToEnd => {
                                self.warnings.push(Warning {
                                    message: "<-> END will not return (just end execution)"
                                        .to_string(),
                                    file: self.file_path.clone(),
                                    line,
                                });
                                BlockType::GoToEnd
                            }
                            _ => resolved_type, // Should not happen
                        }
                    } else {
                        resolved_type
                    };
                }
                Err(e) => {
                    // Collect the error
                    self.errors.push(e);
                }
            }

            // Check for unreachable code after this block
            // Only for GoToSection (->), not GoToSectionAndBack (<->)
            if !is_call_and_back {
                self.detect_unreachable_code(&context.database, block_id);
            }
        }

        Ok(())
    }

    /// Build the full hierarchical path for a section during parsing
    ///
    /// Uses last_section_at_level to build the path from parent sections.
    /// This is called before the section block is added to the database.
    fn build_section_path_during_parse(
        &self,
        database: &Database,
        section_name: &str,
        level: usize,
    ) -> String {
        let mut path_parts = Vec::new();

        // Walk up the section hierarchy using last_section_at_level
        for i in 0..level {
            if i < self.last_section_at_level.len() {
                let section_block_id = self.last_section_at_level[i];
                if let BlockType::Section(section_id) = database.blocks[section_block_id].block_type
                {
                    let section = &database.sections[section_id];
                    let section_name = &database.strings[section.name];
                    path_parts.push(section_name.clone());
                }
            }
        }

        // Add this section's name
        path_parts.push(section_name.to_string());

        // Join with " \ "
        path_parts.join(" \\ ")
    }

    /// Build the full hierarchical id path for a section during parsing
    ///
    /// Uses last_section_at_level to build the id path from parent sections.
    /// This is called before the section block is added to the database.
    fn build_section_id_path_during_parse(
        &self,
        database: &Database,
        section_id: &str,
        level: usize,
    ) -> String {
        let mut path_parts = Vec::new();

        // Walk up the section hierarchy using last_section_at_level
        for i in 0..level {
            if i < self.last_section_at_level.len() {
                let section_block_id = self.last_section_at_level[i];
                if let BlockType::Section(sec_id) = database.blocks[section_block_id].block_type {
                    let section = &database.sections[sec_id];
                    let section_id_name = &database.strings[section.id];
                    path_parts.push(section_id_name.clone());
                }
            }
        }

        // Add this section's id
        path_parts.push(section_id.to_string());

        // Join with " \ "
        path_parts.join(" \\ ")
    }

    /// Build a registry mapping section paths to SectionIds
    fn build_section_registry(&self, database: &Database) -> HashMap<String, SectionId> {
        let mut registry = HashMap::new();

        for block in database.blocks.iter() {
            if let BlockType::Section(section_id) = &block.block_type {
                // Get the path from the section metadata
                let section = &database.sections[*section_id];
                let path = &database.strings[section.id_path];
                registry.insert(path.clone(), *section_id);
            }
        }

        registry
    }

    /// Build the full hierarchical path string for a section block
    ///
    /// Walks up the parent chain to construct a path like "Root \ Child \ Grandchild".
    /// Used for building the section registry and error messages.
    /// Returns an empty string if the block is not a section.
    fn build_section_path_string(&self, database: &Database, block_id: BlockId) -> String {
        let mut path_parts = Vec::new();
        let mut current_id = block_id;

        // Walk up the parent chain, collecting section ids
        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if let BlockType::Section(section_id) = &database.blocks[current_id].block_type {
                let section = &database.sections[*section_id];
                let id_name = &database.strings[section.id];
                path_parts.push(id_name.clone());
            }
            current_id = parent_id;
        }

        // Reverse to get top-down order
        path_parts.reverse();
        path_parts.join(" \\ ")
    }

    /// Find the nearest ancestor section that contains this block
    ///
    /// Walks up the parent chain until a Section block is found.
    /// Returns None if the block is at the root level (no containing section).
    fn find_containing_section(&self, database: &Database, block_id: BlockId) -> Option<BlockId> {
        let mut current_id = block_id;

        if matches!(
            database.blocks[current_id].block_type,
            BlockType::Section { .. }
        ) {
            return Some(current_id);
        }

        // Walk up parents until we find a Section block
        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if matches!(
                database.blocks[parent_id].block_type,
                BlockType::Section { .. }
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
    }

    /// Resolve a go-to-section path to a target BlockId
    ///
    /// This function handles several path formats:
    /// - Absolute paths: `"Root \ Child"` - searches from the root
    /// - Relative paths: `"Child"` or `"Sibling"` - searches children first, then siblings
    /// - Parent navigation: `".."` - navigates to containing section's parent
    /// - Current section: `"."` - refers to the containing section itself
    /// - Multi-level parent: `".. \ .."` - navigates up multiple levels
    /// - Combined: `".. \ Sibling"` - goes to parent, then to sibling
    ///
    /// Search order for relative paths:
    /// 1. Children of containing section
    /// 2. Siblings of containing section
    /// 3. Check full registry for absolute match
    ///
    /// Helper to extract SectionId from a section BlockId
    fn get_section_id(&self, database: &Database, block_id: BlockId) -> Option<SectionId> {
        if let BlockType::Section(section_id) = database.blocks[block_id].block_type {
            Some(section_id)
        } else {
            None
        }
    }

    /// Returns the BlockType variant for the resolved goto, or an error if not found
    fn resolve_path(
        &mut self,
        path: &str,
        containing_section: Option<BlockId>,
        registry: &HashMap<String, SectionId>,
        database: &Database,
        line: usize,
    ) -> Result<BlockType, ParseError> {
        let path = path.trim();

        // Handle special keywords
        match path {
            "START" => return Ok(BlockType::GoToStart),
            "RESTART" => return Ok(BlockType::GoToRestart),
            "END" => return Ok(BlockType::GoToEnd),
            _ => {}
        }

        // Handle "." (current section)
        if path == "." {
            if let Some(section_block_id) = containing_section {
                // Get the SectionId from the block
                if let BlockType::Section(section_id) = database.blocks[section_block_id].block_type
                {
                    return Ok(BlockType::GoTo(section_id));
                }
            }
            return Err(ParseError::SectionNotFound {
                path: path.to_string(),
                file: self.file_path.clone(),
                line,
            });
        }

        // Parse the path into segments
        let segments: Vec<&str> = path.split(" \\ ").map(|s| s.trim()).collect();

        // Check if this is an absolute path (doesn't start with ..)
        if !segments[0].starts_with("..") {
            // Try absolute path first
            if let Some(&section_id) = registry.get(path) {
                return Ok(BlockType::GoTo(section_id));
            }

            // Try relative path (search children and siblings)
            if let Some(section_block_id) = containing_section {
                // Search children first
                if let Some(child_block_id) =
                    self.find_child_section(database, section_block_id, segments[0])
                {
                    if segments.len() == 1 {
                        if let Some(section_id) = self.get_section_id(database, child_block_id) {
                            return Ok(BlockType::GoTo(section_id));
                        }
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&section_id) = registry.get(&full_path) {
                        return Ok(BlockType::GoTo(section_id));
                    }
                }

                // Search siblings
                if let Some(sibling_block_id) =
                    self.find_sibling_section(database, section_block_id, segments[0])
                {
                    if segments.len() == 1 {
                        if let Some(section_id) = self.get_section_id(database, sibling_block_id) {
                            return Ok(BlockType::GoTo(section_id));
                        }
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&section_id) = registry.get(&full_path) {
                        return Ok(BlockType::GoTo(section_id));
                    }
                }
            }
        } else {
            // Handle ".." navigation
            let mut current_section = containing_section;
            let mut segment_index = 0;

            // Process ".." segments
            while segment_index < segments.len() && segments[segment_index] == ".." {
                if let Some(section_id) = current_section {
                    // Navigate to parent section
                    current_section = self.find_parent_section(database, section_id);
                    if current_section.is_none() {
                        return Err(ParseError::NavigationAboveRoot {
                            file: self.file_path.clone(),
                            line,
                        });
                    }
                } else {
                    return Err(ParseError::NavigationAboveRoot {
                        file: self.file_path.clone(),
                        line,
                    });
                }
                segment_index += 1;
            }

            // If there are more segments, resolve them
            if segment_index < segments.len() {
                if let Some(section_block_id) = current_section {
                    let remaining_path = segments[segment_index..].join(" \\ ");

                    // 1) Prefer resolving as a child path from the resolved parent
                    let current_path = self.build_section_path_string(database, section_block_id);
                    let child_full_path = if current_path.is_empty() {
                        remaining_path.clone()
                    } else {
                        format!("{} \\ {}", current_path, remaining_path)
                    };
                    if let Some(&section_id) = registry.get(&child_full_path) {
                        return Ok(BlockType::GoTo(section_id));
                    }

                    // 2) If not found, try as a sibling path (relative to parent section)
                    let parent_path = self
                        .find_parent_section(database, section_block_id)
                        .map(|parent_id| self.build_section_path_string(database, parent_id))
                        .unwrap_or_default();
                    let sibling_full_path = if parent_path.is_empty() {
                        remaining_path.clone()
                    } else {
                        format!("{} \\ {}", parent_path, remaining_path)
                    };
                    if let Some(&section_id) = registry.get(&sibling_full_path) {
                        return Ok(BlockType::GoTo(section_id));
                    }
                }
            } else {
                // Just "..", return the parent section
                if let Some(section_block_id) = current_section {
                    if let Some(section_id) = self.get_section_id(database, section_block_id) {
                        return Ok(BlockType::GoTo(section_id));
                    }
                }
            }
        }

        Err(ParseError::SectionNotFound {
            path: path.to_string(),
            file: self.file_path.clone(),
            line,
        })
    }

    /// Find a direct child section of the given parent by display name
    ///
    /// Searches only immediate children, not nested descendants.
    /// Returns the BlockId of the matching section, or None if not found.
    fn find_child_section(
        &self,
        database: &Database,
        parent_id: BlockId,
        name: &str,
    ) -> Option<BlockId> {
        for &child_id in &database.blocks[parent_id].children {
            if let BlockType::Section(section_id) = &database.blocks[child_id].block_type {
                let section = &database.sections[*section_id];
                let section_id_name = &database.strings[section.id];
                if section_id_name == name {
                    return Some(child_id);
                }
            }
        }
        None
    }

    /// Find a sibling section (shares same parent) by display name
    ///
    /// Searches all children of the given section's parent, excluding the section itself.
    /// Returns the BlockId of the matching sibling, or None if not found or if section has no parent.
    fn find_sibling_section(
        &self,
        database: &Database,
        section_id: BlockId,
        name: &str,
    ) -> Option<BlockId> {
        if let Some(parent_id) = database.blocks[section_id].parent_id {
            for &sibling_id in &database.blocks[parent_id].children {
                if sibling_id != section_id {
                    if let BlockType::Section(sec_id) = &database.blocks[sibling_id].block_type {
                        let section = &database.sections[*sec_id];
                        let section_id_name = &database.strings[section.id];
                        if section_id_name == name {
                            return Some(sibling_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the parent section of a given section
    ///
    /// Walks up the parent chain to find the nearest ancestor that is also a Section block.
    /// Used for resolving ".." navigation. Returns None if the section is at the root.
    fn find_parent_section(&self, database: &Database, section_id: BlockId) -> Option<BlockId> {
        let mut current_id = section_id;

        while let Some(parent_id) = database.blocks[current_id].parent_id {
            if matches!(
                database.blocks[parent_id].block_type,
                BlockType::Section { .. }
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
    }

    /// Validate that section names don't contain backslash and aren't reserved words
    fn validate_section_names(&mut self, database: &Database) -> Result<(), ParseError> {
        for block in database.blocks.iter() {
            if let BlockType::Section(section_id) = &block.block_type {
                let section = &database.sections[*section_id];
                let display_name = &database.strings[section.name];
                let id_name = &database.strings[section.id];
                let skip_id_validation = section.id == section.name;

                let checks = [
                    ("Section name", display_name, false),
                    ("Section id", id_name, skip_id_validation),
                ];

                for (label, name, skip) in checks {
                    if skip {
                        continue;
                    }

                    // Check for reserved words
                    match name.as_str() {
                        "END" | "START" | "RESTART" => {
                            self.errors.push(ParseError::InvalidSectionName {
                                message: format!("{} \"{}\" is reserved", label, name),
                                name: name.clone(),
                                file: self.file_path.clone(),
                                line: block.line,
                            });
                        }
                        _ => {}
                    }

                    // Check for backslash
                    if name.contains('\\') {
                        self.errors.push(ParseError::InvalidSectionName {
                            message: format!("{}s cannot contain '\\' character", label),
                            name: name.clone(),
                            file: self.file_path.clone(),
                            line: block.line,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Detect empty sections
    fn detect_empty_sections(&mut self, database: &Database) -> Result<(), ParseError> {
        for (block_id, block) in database.blocks.iter().enumerate() {
            if let BlockType::Section(section_id) = &block.block_type {
                let section = &database.sections[*section_id];
                let display_name = &database.strings[section.name];

                // Check if this section has any non-section children (recursively)
                let has_content = Self::section_has_content(database, block_id);

                if !has_content {
                    self.errors.push(ParseError::EmptySection {
                        name: display_name.clone(),
                        file: self.file_path.clone(),
                        line: block.line,
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if a section has any content (String, Set, or Goto blocks), recursively
    fn section_has_content(database: &Database, section_id: BlockId) -> bool {
        for &child_id in &database.blocks[section_id].children {
            match &database.blocks[child_id].block_type {
                BlockType::String(_)
                | BlockType::Set(_)
                | BlockType::GoTo(_)
                | BlockType::GoToAndBack(_)
                | BlockType::GoToStart
                | BlockType::GoToRestart
                | BlockType::GoToEnd => return true,
                BlockType::Section(_) if Self::section_has_content(database, child_id) => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    /// Check if a section has only goto blocks (no String blocks), recursively
    fn section_has_only_gotos(database: &Database, section_id: BlockId) -> bool {
        let mut has_any_blocks = false;
        for &child_id in &database.blocks[section_id].children {
            match &database.blocks[child_id].block_type {
                BlockType::String(_) => return false, // Has a string, so not only gotos
                BlockType::GoTo(_)
                | BlockType::GoToAndBack(_)
                | BlockType::GoToStart
                | BlockType::GoToRestart
                | BlockType::GoToEnd => {
                    has_any_blocks = true;
                }
                BlockType::Section(_) => {
                    // For subsections, recursively check
                    if !Self::section_has_only_gotos(database, child_id) {
                        return false; // Subsection has content, so not only gotos
                    }
                    has_any_blocks = true;
                }
                _ => {}
            }
        }
        has_any_blocks // True if we found goto blocks and no string blocks
    }

    /// Detect unreachable code after a GoToSection block
    fn detect_unreachable_code(&mut self, database: &Database, goto_block_id: BlockId) {
        let goto_block = &database.blocks[goto_block_id];

        // Check for sibling blocks after this one
        if let Some(parent_id) = goto_block.parent_id {
            let parent = &database.blocks[parent_id];
            if let Some(pos) = parent.children.iter().position(|&id| id == goto_block_id) {
                // Siblings after this one are unreachable UNLESS they are sections
                // (sections can be jumped to, so they're not unreachable)
                for &sibling_id in &parent.children[pos + 1..] {
                    let sibling = &database.blocks[sibling_id];
                    // Only warn about non-section blocks
                    if !matches!(sibling.block_type, BlockType::Section { .. }) {
                        self.warnings.push(Warning {
                            message: "Unreachable code after section jump".to_string(),
                            file: self.file_path.clone(),
                            line: sibling.line,
                        });
                    }
                }
            }
        }

        // Check for child blocks (they are also unreachable)
        // BUT don't warn about section children - they can be jumped to
        for &child_id in &goto_block.children {
            let child = &database.blocks[child_id];
            if !matches!(child.block_type, BlockType::Section { .. }) {
                self.warnings.push(Warning {
                    message: "Unreachable code after section jump".to_string(),
                    file: self.file_path.clone(),
                    line: child.line,
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cuentitos_common::test_case::TestCase;

    #[test]
    fn test_single_line_script() {
        let test_case = TestCase::from_string(
            include_str!("../../compatibility-tests/strings/feature/single-line-and-end.md"),
            "single-line.md",
        );

        let mut parser = Parser::default();
        let (database, _warnings) = parser.parse(test_case.script).unwrap();

        assert_eq!(database.blocks.len(), 3);
        assert_eq!(database.strings.len(), 1);

        // Check Start block
        assert_eq!(database.blocks[0].block_type, BlockType::Start);
        assert_eq!(database.blocks[0].level, 0);
        assert_eq!(database.blocks[0].parent_id, None);
        assert_eq!(database.blocks[0].children, vec![1]); // Points to the string block

        // Check String block
        if let BlockType::String(id) = database.blocks[1].block_type {
            assert_eq!(database.strings[id], "This is a single line");
        } else {
            panic!("Expected String block");
        }
        assert_eq!(database.blocks[1].level, 0);
        assert_eq!(database.blocks[1].parent_id, Some(0));
        assert!(database.blocks[1].children.is_empty());

        // Check End block
        assert_eq!(database.blocks[2].block_type, BlockType::End);
        assert_eq!(database.blocks[2].level, 0);
        assert_eq!(database.blocks[2].parent_id, None);
        assert!(database.blocks[2].children.is_empty());
    }

    #[test]
    fn test_indented_script() {
        let script = "Parent\n  Child1\n  Child2\n    Grandchild\n  Child3";

        let mut parser = Parser::default();
        let (database, _warnings) = parser.parse(script).unwrap();

        // We expect: Start, Parent, Child1, Child2, Grandchild, Child3, End
        assert_eq!(database.blocks.len(), 7);
        assert_eq!(database.strings.len(), 5);

        // Verify levels
        assert_eq!(database.blocks[1].level, 0); // Parent
        assert_eq!(database.blocks[2].level, 1); // Child1
        assert_eq!(database.blocks[3].level, 1); // Child2
        assert_eq!(database.blocks[4].level, 2); // Grandchild
        assert_eq!(database.blocks[5].level, 1); // Child3

        // Verify parent-child relationships
        assert_eq!(database.blocks[1].parent_id, Some(0)); // Parent -> Start
        assert_eq!(database.blocks[2].parent_id, Some(1)); // Child1 -> Parent
        assert_eq!(database.blocks[3].parent_id, Some(1)); // Child2 -> Parent
        assert_eq!(database.blocks[4].parent_id, Some(3)); // Grandchild -> Child2
        assert_eq!(database.blocks[5].parent_id, Some(1)); // Child3 -> Parent
    }

    #[test]
    fn test_warning_section_whitespace() {
        // Note: "# " takes one space, so "#  " leaves one extra leading space
        // Trailing spaces are preserved in the string literal
        let script = "#  Section A  \nText in A";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 1 warning for the section name
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0]
            .message
            .contains("Section name has leading/trailing whitespace"));
        assert!(warnings[0].message.contains("Section A"));
        assert_eq!(warnings[0].line, 1);
    }

    #[test]
    fn test_warning_unreachable_siblings() {
        let script = "# Section A\nText before\n-> Section B\nText after sibling 1\nText after sibling 2\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 2 warnings for unreachable siblings
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0]
            .message
            .contains("Unreachable code after section jump"));
        assert!(warnings[1]
            .message
            .contains("Unreachable code after section jump"));
        assert_eq!(warnings[0].line, 4); // "Text after sibling 1"
        assert_eq!(warnings[1].line, 5); // "Text after sibling 2"
    }

    #[test]
    fn test_warning_unreachable_children() {
        let script = "# Section A\nText before\n-> Section B\n  Text child 1\n  Text child 2\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have 2 warnings for unreachable children
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0]
            .message
            .contains("Unreachable code after section jump"));
        assert!(warnings[1]
            .message
            .contains("Unreachable code after section jump"));
        assert_eq!(warnings[0].line, 4); // "  Text child 1"
        assert_eq!(warnings[1].line, 5); // "  Text child 2"
    }

    #[test]
    fn test_subsections_not_unreachable() {
        // Subsections should NOT be flagged as unreachable even if there's a goto in the parent
        let script = "# Parent\n-> Child A\n  ## Child A\n  Text in child A\n  ## Child B\n  Text in child B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_ok());
        let (_database, warnings) = result.unwrap();

        // Should have NO warnings - subsections are not unreachable
        assert_eq!(
            warnings.len(),
            0,
            "Subsections should not be flagged as unreachable. Got warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn test_error_section_not_found() {
        let script = "# Section A\n-> Nonexistent Section";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::SectionNotFound { path, line, .. }) => {
                assert_eq!(path, "Nonexistent Section");
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::SectionNotFound { path, line, .. }
                    if path == "Nonexistent Section" && *line == 2)
                ));
            }
            _ => panic!("Expected SectionNotFound error"),
        }
    }

    #[test]
    fn test_error_navigation_above_root() {
        let script = "# Section A\n-> ..";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::NavigationAboveRoot { line, .. }) => {
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors
                    .iter()
                    .any(|e| matches!(e, ParseError::NavigationAboveRoot { line, .. }
                    if *line == 2)));
            }
            _ => panic!("Expected NavigationAboveRoot error"),
        }
    }

    #[test]
    fn test_error_empty_section() {
        // An empty section has no children at all (not even subsections or goto commands)
        let script = "# Section A\n\n# Section B\nText in B";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::EmptySection { name, line, .. }) => {
                assert_eq!(name, "Section A");
                assert_eq!(line, 1);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                // Could have multiple empty section errors
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::EmptySection { name, .. } if name == "Section A")
                ));
            }
            _ => panic!("Expected EmptySection error"),
        }
    }

    #[test]
    fn test_section_with_only_goto_is_not_empty() {
        // Bug: sections with only goto commands should NOT be flagged as empty
        let script =
            "# Section A\n-> Section C\n\n# Section B\nText in B\n\n# Section C\nText in C";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        // Should succeed - sections with goto commands are valid
        assert!(
            result.is_ok(),
            "Section with only goto command should be valid, got error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_jump_to_parent_with_dotdot() {
        // Test that -> .. correctly resolves to parent section
        let script = "# Parent\nText in parent\n  ## Child\n  Text in child\n  -> ..";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        assert!(
            result.is_ok(),
            "Jump to parent using .. should work, got error: {:?}",
            result.err()
        );

        let (database, _warnings) = result.unwrap();

        // Find the GoToSection block
        let goto_block = database
            .blocks
            .iter()
            .find(|b| matches!(b.block_type, BlockType::GoTo(_)));
        assert!(goto_block.is_some(), "Should have a GoTo block");

        // Check that it resolves to the Parent section
        if let Some(block) = goto_block {
            if let BlockType::GoTo(section_id) = block.block_type {
                let section = &database.sections[section_id];
                assert_eq!(
                    section.block_id, 1,
                    "Should resolve to Parent section (block 1)"
                );
            }
        }
    }

    #[test]
    fn test_error_malformed_empty_reference() {
        let script = "# Section A\n->";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 2)
                ));
            }
            Err(e) => panic!("Expected InvalidGoToSection error, got: {:?}", e),
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_malformed_trailing_backslash() {
        let script = "# Section A\n-> Root \\";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section names separated by ' \\\\ '"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section names separated by ' \\\\ '") && *line == 2)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
    }

    #[test]
    fn test_error_malformed_no_space_after_arrow() {
        let script = "# Section A\n->Section A";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section name after '->'"));
                assert_eq!(line, 2);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section name after '->'") && *line == 2)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
    }

    #[test]
    fn test_warning_extra_spaces_in_goto_path() {
        // Extra spaces in goto path should be accepted with a warning (not an error)
        let script = "# Section A\nText\n->  Section A";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        // Should succeed (not error)
        assert!(
            result.is_ok(),
            "Extra spaces in goto path should be accepted, got error: {:?}",
            result.err()
        );

        let (_database, warnings) = result.unwrap();
        // Should have a warning about the extra space
        assert!(!warnings.is_empty(), "Should have warning about whitespace");
        assert!(
            warnings
                .iter()
                .any(|w| w.message.contains("whitespace") || w.message.contains("space")),
            "Should warn about whitespace, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_error_malformed_wrong_spacing_around_backslash() {
        let script = "# Root\n  ## Child\nText\n-> Root\\Child";
        let mut parser = Parser::new();
        let result = parser.parse(script);
        assert!(result.is_err());
        match result {
            Err(ParseError::InvalidGoToSection { message, line, .. }) => {
                assert!(message.contains("Expected section names separated by ' \\\\ '"));
                assert_eq!(line, 4);
            }
            Err(ParseError::MultipleErrors { errors }) => {
                assert!(errors.iter().any(
                    |e| matches!(e, ParseError::InvalidGoToSection { message, line, .. }
                    if message.contains("Expected section names separated by ' \\\\ '") && *line == 4)
                ));
            }
            _ => panic!("Expected InvalidGoToSection error"),
        }
    }

    #[test]
    fn test_invalid_indentation() {
        let script = "First line\n   Invalid indentation";
        let mut parser = Parser::new();
        match parser.parse(script) {
            Err(ParseError::InvalidIndentation {
                message: _,
                file: None,
                line: 2,
            }) => (),
            _ => panic!("Expected InvalidIndentation error"),
        }
    }

    #[test]
    fn test_skip_empty_lines() {
        let script = "First\n\n  Second";

        let mut parser = Parser::default();
        let (database, _warnings) = parser.parse(script).unwrap();

        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First");
        assert_eq!(database.strings[1], "Second");
    }

    #[test]
    fn test_invalid_indentation_with_file() {
        let script = "First line\n   Invalid indentation";
        let mut parser = Parser::with_file(PathBuf::from("test.cuentitos"));
        match parser.parse(script) {
            Err(ParseError::InvalidIndentation {
                message: _,
                file: Some(path),
                line: 2,
            }) => {
                assert_eq!(path, PathBuf::from("test.cuentitos"));
            }
            _ => panic!("Expected InvalidIndentation error with file path"),
        }
    }

    #[test]
    fn test_is_comment() {
        assert!(Parser::is_comment("// This is a comment"));
        assert!(Parser::is_comment("  // Indented comment"));
        assert!(Parser::is_comment("    // More indentation"));
        assert!(Parser::is_comment("//"));
        assert!(Parser::is_comment("/// Triple slash"));
        assert!(Parser::is_comment("//// Quadruple slash"));
        assert!(!Parser::is_comment("Not a comment"));
        assert!(!Parser::is_comment("# Section"));
        assert!(!Parser::is_comment("Text with // in middle"));
    }

    #[test]
    fn test_comment_parsing() {
        let script =
            "// Comment at start\nFirst line\n// Comment in middle\nSecond line\n// Comment at end";
        let mut parser = Parser::new();
        let (database, _warnings) = parser.parse(script).unwrap();

        // Should have Start, two string blocks, and End
        assert_eq!(database.blocks.len(), 4);
        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First line");
        assert_eq!(database.strings[1], "Second line");
    }

    #[test]
    fn test_comment_with_arbitrary_indentation() {
        let script = "First line\n      // Comment with weird indentation\nSecond line";
        let mut parser = Parser::new();
        let (database, _warnings) = parser.parse(script).unwrap();

        // Should have Start, two string blocks, and End (comment ignored)
        assert_eq!(database.blocks.len(), 4);
        assert_eq!(database.strings.len(), 2);
        assert_eq!(database.strings[0], "First line");
        assert_eq!(database.strings[1], "Second line");
    }

    #[test]
    fn test_option_parsing_basic() {
        let script = "Choose one\n  * Option A\n    Content A\n  * Option B\n    Content B";
        let mut parser = Parser::new();
        let (database, _warnings) = parser.parse(script).unwrap();

        // Should have: Start, "Choose one", Option A, Content A, Option B, Content B, End
        assert_eq!(database.blocks.len(), 7);

        // Check that we have option blocks
        match &database.blocks[2].block_type {
            BlockType::Option(string_id) => {
                assert_eq!(database.strings[*string_id], "Option A");
            }
            _ => panic!("Expected Option block"),
        }

        match &database.blocks[4].block_type {
            BlockType::Option(string_id) => {
                assert_eq!(database.strings[*string_id], "Option B");
            }
            _ => panic!("Expected Option block"),
        }
    }

    #[test]
    fn test_options_without_parent_at_root() {
        let script = "* Option A\n  Content A\n* Option B\n  Content B";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        // Debug: print what we got
        if let Ok((ref db, _)) = result {
            eprintln!("Database blocks:");
            for (i, block) in db.blocks.iter().enumerate() {
                eprintln!("  {}: {:?}", i, block.block_type);
            }
        }

        assert!(result.is_err(), "Expected error but got success");
        match result.unwrap_err() {
            ParseError::MultipleErrors { errors } => {
                assert_eq!(errors.len(), 2);
                // Both options should error
                for (i, err) in errors.iter().enumerate() {
                    match err {
                        ParseError::OptionsWithoutParent { file: _, line } => {
                            // First option at line 1, second at line 3
                            assert_eq!(*line, if i == 0 { 1 } else { 3 });
                        }
                        _ => panic!("Expected OptionsWithoutParent error"),
                    }
                }
            }
            ParseError::OptionsWithoutParent { file: _, line } => {
                assert_eq!(line, 1); // Single error case
            }
            err => panic!("Expected OptionsWithoutParent error, got: {:?}", err),
        }
    }

    #[test]
    fn test_options_with_non_option_sibling_before() {
        let script = "Parent\n  Regular text\n  * Option A\n    Content A";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::OptionsWithoutParent { file: _, line } => {
                assert_eq!(line, 3); // Option is at line 3
            }
            _ => panic!("Expected OptionsWithoutParent error"),
        }
    }

    #[test]
    fn variables_block_skip_accounting_preserves_content_line_numbers() {
        // After consuming a `--- variables` block, errors raised on later
        // content lines must report their original line number, not an
        // offset into the post-block region.
        let script = "--- variables\nint a = 1\n---\nFirst line\n   Invalid indentation";
        let mut parser = Parser::new();
        let result = parser.parse(script);

        match result {
            Err(ParseError::InvalidIndentation { line, .. }) => {
                assert_eq!(line, 5, "indentation error must point at line 5");
            }
            other => panic!("expected InvalidIndentation at line 5, got {:?}", other),
        }
    }

    #[test]
    fn mid_file_variables_block_emits_warning() {
        // `--- variables` is only valid as the first directive. A later
        // occurrence must surface a warning at its actual line number.
        let script = "Some content\n\n--- variables\nint a = 1\n---";
        let mut parser = Parser::new();
        let (_database, warnings) = parser.parse(script).expect("parse should succeed");

        let matched = warnings.iter().find(|w| {
            w.message
                .contains("'--- variables' block must appear at the start")
        });
        let warning = matched.expect("expected mid-file '--- variables' warning");
        assert_eq!(warning.line, 3, "warning should point at line 3");
    }
}
