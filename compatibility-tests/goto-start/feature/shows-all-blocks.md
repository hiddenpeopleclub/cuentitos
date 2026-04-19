# Jump to START Shows All Blocks

This test verifies that after -> START, continuing execution re-shows all content blocks.

## Script
```cuentitos
First line
Second line
-> START
```

## Input
```input
n,n,n,n,n,n,n,n,n,n,q
```

## Result
```result
START
First line
Second line
START
First line
Second line
START
First line
Second line
QUIT
```
