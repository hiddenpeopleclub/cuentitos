# Set Inside Nested Blocks and Sections

A `set` statement is valid anywhere a regular block is valid, including
inside sections and nested indented blocks.

## Script
```cuentitos
--- variables
string name = "Aria"
---
# chapter_one: Chapter One
set name = "Brenn"
First line
  ## sub: Sub
  set name = "Cass"
  Sub line
```

## Input
```input
n
n
n
n
?
s
```

## Result
```result
START
-> Chapter One
First line
-> Chapter One \ Sub
Sub line
name: "Cass"
END
```
