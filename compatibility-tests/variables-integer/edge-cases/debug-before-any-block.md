# Edge Case: `?` With No `---` Block in the Script

A script with no `--- variables` block should still accept `?` mid-execution.
`?` prints nothing (because no variables are declared) and does not advance the cursor.

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
Second line.
Third line.
END
```
