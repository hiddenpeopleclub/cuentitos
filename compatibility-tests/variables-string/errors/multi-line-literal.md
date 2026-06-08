# Error: String Literal Spanning Multiple Lines

String literals may not span multiple source lines. Because the parser is
line-based, it sees only an opening quote with no closing quote on the
declaration line and reports the same diagnostic as a never-closed
literal — pointing at the opening line. The later closing quote is never
reached.

## Script
```cuentitos
--- variables
string name = "line one
line two"
---

This is the story.
```

## Input
```input
s
```

## Result
```result
multi-line-literal.cuentitos:2: ERROR: Unterminated string literal.
```
