# Error: String Literal Spanning Multiple Lines

String literals may not span multiple source lines. Because the parser is
line-based, it sees only an opening quote with no closing quote on the
declaration line and reports the same diagnostic as a never-closed
literal — pointing at the opening line. The later closing quote is never
reached.

**Assertion note.** This expectation depends on the parser detecting the
unclosed quote on the opening declaration line (line 2) and reporting
`Unterminated string literal` there, *before* the trailing `line two"`
on line 3 is examined. An implementation that instead reached line 3
first would report a different diagnostic (e.g. a malformed declaration
at line 3); the string parser must perform the unterminated-literal check
on the declaration line so this test holds.

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
