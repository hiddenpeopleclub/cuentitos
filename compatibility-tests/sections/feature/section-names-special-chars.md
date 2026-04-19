# Section Names with Special Characters and Unicode

This test verifies that section names can contain special characters and unicode.

## Script
```cuentitos
# Section-1
Text in section 1
-> café

# café
Text in café
-> Section (with parens)

# Section (with parens)
Text with parens
```

## Input
```input
s
```

## Result
```result
START
-> Section-1
Text in section 1
-> café
Text in café
-> Section (with parens)
Text with parens
END
```
