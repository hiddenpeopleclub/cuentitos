# Edge Case: `?` With No `---` Block in the Script

A script with no `--- variables` block should still accept `?` mid-execution.
`?` does not advance the cursor; because no variables are declared, the runtime
emits a warning. Line 0 is used because `?` is a CLI input with no corresponding
script line.

## Script
```cuentitos
First line.
Second line.
Third line.
```

## Input
```input
n
?
s
```

## Result
```result
START
First line.
debug-before-any-block.cuentitos:0: WARNING: No variables declared.
Second line.
Third line.
END
```
