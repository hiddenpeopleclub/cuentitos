# Error: Unclosed Variables Block

A variables block without a closing `---` is a parse-time error.

## Script
```cuentitos
--- variables
int score
Hello
```

## Input
```input
s
```

## Result
```result
Error: Unclosed variables block
```
