# Set Inside Nested Blocks and Sections

A `set` statement is valid anywhere a regular block is valid, including inside
sections and nested indented blocks.

## Script
```cuentitos
--- variables
int score = 0
---
# chapter_one: Chapter One
set score = 10
First line
  ## sub: Sub
  set score = 20
  Sub line
```

## Input
```input
n
n
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
score: 20
END
```
