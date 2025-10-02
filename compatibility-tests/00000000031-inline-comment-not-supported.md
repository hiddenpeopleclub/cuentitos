# Inline Comments Not Supported

Comments must be on their own line. Text with // in the middle is treated as literal text, not as a comment.

## Script
```cuentitos
This is text // this is NOT a comment
Another line with // symbols in the middle
```

## Input
```input
s
```

## Result
```result
START
This is text // this is NOT a comment
Another line with // symbols in the middle
END
```
