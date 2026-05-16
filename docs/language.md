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

Text blocks can be nested using indentation. The indentation must be exactly 2 spaces per level. Using any other number of spaces (like 3) will give an error.

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
   - The `section_id` is the short name used to jump to or refer to the section from elsewhere in the script
   - The `Display Name` is what the reader sees

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

### What the Reader Sees

When the story moves into a section, an entry message is shown with the path through the section hierarchy:

- Example: `Entered Section: First Section > Subsection (section_1/subsection_1)`

Sections will be the foundation for future navigation features like "Go To Section" and menu systems.

## Variables

Variables let your story remember things — a hero's health, how many coins the player has, whether a door is open. Cuentitos variables hold whole numbers (no decimals).

### Declaring Variables

Every variable used in a story must first be listed in a `--- variables` block at the top of the script, before any story content. Each line inside the block names one variable, with the keyword `int` (short for "integer", meaning whole number):

```cuentitos
--- variables
int health
int score = 0
int max_health = 100
---

The adventure begins.
```

- `int <name>` — creates a variable that starts at `0`.
- `int <name> = <starting value>` — creates a variable with the starting value you choose.

Variable names can contain letters, digits, and underscores. The words `and`, `or`, and `not` are reserved for conditions (see `req` below) and cannot be used as variable names.

### Starting Values

A starting value can be a number, a small piece of math (using `+ - * /` and parentheses), or another variable from earlier in the same block. Starting values are worked out once, when the story loads, so any mistake — like dividing by zero or going past the largest allowed number — is caught and shown to you before the story starts.

```cuentitos
--- variables
int a = 5
int b = a + 5
int c = (a + b) * 2
---
```

You can only refer to variables that were already declared above. A variable cannot be used in its own line or in any line before its own declaration.

Negative numbers are allowed:

```cuentitos
--- variables
int penalty = -10
int adjusted = -(penalty + 5)
---
```

An empty `--- variables` block is valid and declares no variables.

### Checking Variable Values

While running a story in the CLI, type `?` to see the current value of every declared variable, in the order they were declared:

```
START
health: 100
score: 0
END
```

### The `set` Statement

`set` changes a variable's value while the story is playing. It can appear anywhere a normal line can — at the top level, inside a section, or under another block.

**Simple assignment:**

```cuentitos
--- variables
int health = 10
---
set health = 5
You are wounded.
```

**Math on the right side:**

The right side of a `set` can use `+`, `-`, `*`, `/`, parentheses, numbers, and any declared variable — including the one being changed:

```cuentitos
--- variables
int score = 0
int bonus = 3
---
set score = (score + bonus) * 2
```

Math follows the usual rules: `*` and `/` happen before `+` and `-`, and parentheses change the order.

**Shortcuts for common changes:**

| Form | What it does |
| --- | --- |
| `set x = value` | Sets `x` to `value` |
| `set x += value` | Adds `value` to `x` |
| `set x -= value` | Subtracts `value` from `x` |
| `set x *= value` | Multiplies `x` by `value` |
| `set x /= value` | Divides `x` by `value` |

```cuentitos
--- variables
int score = 10
---
set score += 5
set score *= 2
```

`set` only assigns values — it cannot use comparison operators like `=` (equal), `<`, or `>`. Those belong in `req` (see below).

**A note on division:** because variables hold only whole numbers, division drops the fractional part toward zero. So `7 / 2` is `3` (not `3.5`), and `-7 / 2` is `-3` (not `-4`).

`set` is worked out while the story is playing, against the variable's current value. If something goes wrong on a `set` line — like dividing by zero or going past the largest allowed number — the story stops and reports the error on that line.

### The `req` Statement

`req` (short for "require") puts a condition on the block it sits under. If the condition is false, that block — and everything inside it — is skipped.

`req` must be indented underneath the block it guards. It cannot stand on its own at the top level of the script.

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

Only `The door is open.` is shown, because `door_open` is `1`.

When a guarded block has children of its own, the whole branch is skipped if the `req` is false:

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

Each side of a comparison can be math (numbers, variables, `+ - * /`, parentheses), with the usual order of operations.

**Several `req` lines under the same block — all must pass:**

When a block has more than one `req` underneath it, every one of them must be true for the block to show.

```cuentitos
--- variables
int x = 5
---

In range.
  req x > 0
  req x < 10
```

**`req` reflects the latest values:**

A `req` is checked against the variable's current value at the moment that block is reached. A `set` earlier in the story changes what later `req`s see:

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

### Combining Conditions: `and`, `or`, `not`

A single `req` can combine several conditions using `and`, `or`, and `not`. These words must be lowercase — `AND`, `OR`, `NOT` are treated as variable names and will give an "undefined variable" error.

```cuentitos
--- variables
int health = 10
int shield = 0
---

Alive but exposed.
  req health > 0 and not shield > 0
```

**Order conditions are read in (strongest first):**

1. `not` — flips a single condition
2. `and` — both sides must be true
3. `or` — at least one side must be true

So `a or b and c` is read as `a or (b and c)`, and `not a and b` is read as `(not a) and b`.

Use parentheses to group conditions your own way:

```cuentitos
--- variables
int a = 1
int b = 0
int c = 0
---

With explicit grouping.
  req (a > 0 or b > 0) and c > 0
```

Without the parentheses, `a > 0 or b > 0 and c > 0` would be read as `a > 0 or (b > 0 and c > 0)` and would be true (because `a > 0` is true). With them, the `and c > 0` applies to the whole `or`, which makes the line false.

**Conditions stop being checked once the answer is known:**

In `a or b`, if `a` is already true, `b` isn't checked. In `a and b`, if `a` is already false, `b` isn't checked. The same goes for branches: if a parent block's `req` fails, none of its children's `req`s are checked at all.

**Mixing `and`/`or` on one line with several `req` lines:**

An `and`/`or` line counts as one condition. If a block has both kinds, every line must still be true:

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

### When Errors Are Reported

Some mistakes are caught when the story loads, before any text is shown. Others can only be caught while the story is playing, because they depend on values that change.

| Situation | When it's reported |
| --- | --- |
| Dividing by zero in a starting value | When the story loads |
| Going past the largest allowed number in a starting value | When the story loads |
| Dividing by zero in a `set` or `req` | While playing |
| Going past the largest allowed number in a `set` or `req` | While playing |
| Using a variable that was never declared | When the story loads |
| Using a variable in a starting value before it's declared | When the story loads |
| Declaring two variables with the same name | When the story loads |
| Using a reserved word (`and`, `or`, `not`) as a variable name | When the story loads |
| `req` at the top level (with no block above it) | When the story loads |

Variables hold whole numbers between `-9223372036854775808` and `9223372036854775807`. Going past either end is what "the largest allowed number" refers to in the table above.
