# Set Inside Nested Blocks and Sections

A `set` on an enum variable is valid anywhere a regular block is valid,
including inside sections and nested indented blocks. Each `set` executes
silently as the runtime enters the block.

## Script
```cuentitos
--- variables
enum status = idle, active, done
---
# chapter_one: Chapter One
set status = active
First line
  ## sub: Sub
  set status = done
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
status: done
END
```
