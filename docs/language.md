# Cuentitos Language Documentation

This document describes the currently implemented features of the Cuentitos language based on the compatibility tests.

## Basic Structure

### Text Blocks

The most basic element in Cuentitos is a text block. Text blocks are simply lines of text that will be displayed in sequence:

```cuentitos
This is a single line of text
This is another line of text
```

### Indentation and Nesting

Text blocks can be nested using indentation. The indentation must be exactly 2 spaces per level. Using any other number of spaces (like 3) will result in a parse error.

Example of correct nesting:

```cuentitos
This is a parent string
  This is a child string
    This is a grandchild string
  This is another child string
```

Example that would cause an error:

```cuentitos
First line
   Invalid indentation  # Error: 3 spaces instead of 2
```

Nesting creates parent-child relationships between blocks, which affects navigation and flow control.

## Sections

Sections organize content into navigable segments using markdown-style headers with custom IDs:

```cuentitos
# section_1: First Section
  Content within the first section

## subsection_1: First Subsection
  Content within a subsection

### deep_section: Deeply Nested Section
  Content at the third level

## subsection_2: Second Subsection
  More content here

# section_2: Second Section
  Content in the second section
```

### Section Syntax Rules

1. **Format**: `# section_id: Display Name`
   - The `section_id` is used for programmatic reference (navigation, jumps)
   - The `Display Name` is shown to users

2. **Hierarchy Levels**:
   - `#` = Top-level section (level 0)
   - `##` = Subsection (level 1)
   - `###` = Sub-subsection (level 2)
   - And so on...

3. **Content Indentation**:
   - Content following a section header **must** be indented with at least 2 spaces
   - Sections can follow other sections without additional indentation

4. **Section Nesting**:
   - Sections automatically nest based on their header level
   - A `##` section becomes a child of the nearest preceding `#` section
   - A `###` section becomes a child of the nearest preceding `##` section

Example with error:
```cuentitos
# section_1: First Section
Some content without indentation  # ERROR: Content must be indented under its section
```

Correct version:
```cuentitos
# section_1: First Section
  Some content with proper indentation  # OK: Content is indented
```

### Runtime Behavior

When navigating through sections, the runtime displays:
- Section entry messages showing the hierarchy path
- Example: `Entered Section: First Section > Subsection (section_1/subsection_1)`

Sections will be essential for future navigation features like "Go To Section" and menu systems

## Variables

### Declaring Variables

Variables are declared in a `--- variables` block that must appear at the top of the script, before any story content. Each line inside the block declares one variable using the `int` keyword:

```cuentitos
--- variables
int health
int score = 0
int max_health = 100
---

The adventure begins.
```

- `int <name>` — declares a variable with a default of `0`.
- `int <name> = <expr>` — declares a variable with a computed default.

Variable names may contain letters, digits, and underscores. The names `and`, `or`, and `not` are reserved and cannot be used.

### Default Values

Default expressions are evaluated at **parse time**, not at runtime. They may reference literals, arithmetic operators (`+ - * /`), parentheses, and variables declared **earlier** in the same block. Forward references (referencing a name declared later) are a parse-time error.

```cuentitos
--- variables
int a = 5
int b = a + 5
int c = (a + b) * 2
---
```

Negative literals are allowed:

```cuentitos
--- variables
int penalty = -10
int adjusted = -(penalty + 5)
---
```

Because defaults are evaluated at parse time, division by zero or integer overflow in a default expression is reported immediately as a parse-time error.

An empty `--- variables` block is valid and declares no variables.

### Querying Variables

At the CLI, send `?` to print the current value of all declared variables in declaration order:

```
START
health: 100
score: 0
END
```

### The `set` Statement

`set` assigns a new value to a declared variable at runtime. It can appear anywhere a regular block can — at the top level, inside sections, or inside indented blocks.

**Simple assignment:**

```cuentitos
--- variables
int health = 10
---
set health = 5
You are wounded.
```

**Arithmetic on the right-hand side:**

The RHS may use `+`, `-`, `*`, `/`, parentheses, integer literals, and references to any declared variable (including the variable being assigned):

```cuentitos
--- variables
int score = 0
int bonus = 3
---
set score = (score + bonus) * 2
```

Standard arithmetic precedence applies: `*` and `/` bind tighter than `+` and `-`. Parentheses override precedence.

**Compound assignment operators:**

| Form | Meaning |
| --- | --- |
| `set x = expr` | Assign `expr` to `x` |
| `set x += expr` | Add `expr` to `x` |
| `set x -= expr` | Subtract `expr` from `x` |
| `set x *= expr` | Multiply `x` by `expr` |
| `set x /= expr` | Divide `x` by `expr` (truncates toward zero) |

```cuentitos
--- variables
int score = 10
---
set score += 5
set score *= 2
```

`set` is an assignment statement only; comparison operators are not valid on the RHS.

Integer division truncates toward zero: `-7 / 2` is `-3`, not `-4`.

`set` expressions are always evaluated at runtime against the variable's current value, even when the RHS contains only constants. Runtime errors (division by zero, overflow) are reported on the line where the `set` appears.

### The `req` Statement

`req` (short for "require") gates the block it is a child of. If the condition is false, the parent block and all its descendants are skipped.

`req` must be indented under the block it guards — it cannot appear at the top level of the script.

```cuentitos
--- variables
int door_open = 1
---

You approach the house.
The door is open.
  req door_open = 1
The door is locked.
  req door_open = 0
You step inside.
```

Only `The door is open.` is shown because `door_open` equals `1`.

When a gated block has children of its own, the entire subtree is skipped when the `req` fails:

```cuentitos
--- variables
int visited = 0
---

You stand in the hallway.
The old house.
  req visited = 1
  You remember being here.
  The smell is familiar.
You leave the hallway.
```

**Comparison operators:**

| Operator | Meaning |
| --- | --- |
| `=` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less than or equal |
| `>` | Greater than |
| `>=` | Greater than or equal |

The LHS and RHS of a comparison may each be a full arithmetic expression (literals, variables, `+ - * /`, parentheses). Standard arithmetic precedence applies within each side.

**Multiple `req` siblings — implicit AND:**

When a block has more than one `req` child, all of them must pass for the parent to be shown. This is an implicit AND across sibling `req`s:

```cuentitos
--- variables
int x = 5
---

In range.
  req x > 0
  req x < 10
```

**`req` and `set` interact at runtime:**

`req` conditions are evaluated against the variable's current value at the moment that block is reached. A `set` earlier in the script changes what subsequent `req`s see:

```cuentitos
--- variables
int flag = 0
---

Before set, flag is zero.
  req flag = 0
set flag = 1
After set, flag is one.
  req flag = 1
```

### Logical Operators in `req`

A single `req` can combine multiple conditions using `and`, `or`, and `not`. These keywords are lowercase only — `AND`, `OR`, `NOT` are treated as variable names and will produce an "undefined variable" error.

```cuentitos
--- variables
int health = 10
int shield = 0
---

Alive but exposed.
  req health > 0 and not shield > 0
```

**Precedence (tightest to loosest):**

1. `not` — prefix negation of a comparison
2. `and` — logical conjunction
3. `or` — logical disjunction

So `a or b and c` parses as `a or (b and c)`, and `not a and b` parses as `(not a) and b`.

Use parentheses to override the default grouping:

```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

With explicit grouping.
  req (a > 0 or b > 0) and c > 0
```

Without the parentheses, `a > 0 or b > 0 and c > 0` would parse as `a > 0 or (b > 0 and c > 0)` and evaluate to true (because `a > 0` is true); with them, the `and c > 0` applies to the whole `or` sub-expression, which makes it false.

**Short-circuit evaluation:**

`and` and `or` short-circuit: the right operand is not evaluated when the result is already determined by the left. Similarly, when a parent block's `req` fails, none of the child blocks' `req` expressions are evaluated.

**Combining inline `and`/`or` with sibling `req`s:**

An inline logical expression counts as one condition in the implicit sibling AND. Both the inline expression and any sibling `req`s must pass:

```cuentitos
--- variables
int health = 10
int mana = 5
int shield = 0
---

Both conditions pass.
  req health > 0 or shield > 0
  req mana > 0
```

### Errors

| Situation | When reported |
| --- | --- |
| Division by zero in a default expression | Parse time |
| Integer overflow in a default expression | Parse time |
| Division by zero in a `set` or `req` expression | Runtime |
| Integer overflow in a `set` or `req` expression | Runtime |
| Undeclared variable referenced in `set` or `req` | Parse time |
| Forward reference in a default expression | Parse time |
| Duplicate variable name | Parse time |
| Reserved keyword used as a variable name | Parse time |
| `req` at the top level (no parent block) | Parse time |

Integers are stored as 64-bit signed values (i64). The maximum value is `9223372036854775807`; the minimum is `-9223372036854775808`.
